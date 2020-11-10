import math
import uuid

from django.db import models
from django.db.models import Sum
from polymorphic.models import PolymorphicModel


class Voting(PolymorphicModel):
    voting_name = models.CharField(max_length=50)
    voting_description = models.CharField(max_length=200, blank=True)
    max_votes = models.IntegerField(default=1)
    is_open = models.BooleanField(default=False)
    is_ended = models.BooleanField(default=False)
    treshold = models.FloatField(default=500.0)
    is_password_protected = models.BooleanField(default=False)
    voting_password = models.CharField(max_length=50, blank=True)

    # For the purpose of getting unique DOM element IDs
    def pseudo_unique_id(self):
        if self.is_transferable:
            return f"{self.id}-transferable"
        return f"{self.id}-regular"

    # Returns anything only after the voting has been closed
    def voter_statuses(self):
        if self.is_transferable:
            return TransferableVotingVoterStatus.objects.all().filter(voting=self)
        return NormalVotingVoterStatus.objects.all().filter(voting=self)

    def total_votes(self):
        if self.is_open:
            return int(math.floor(self.vote_set.count() / self.max_votes))
        else:
            result = self.voting_results.aggregate(sum=Sum("vote_count"))
            if result:
                return int(math.floor(result.get("sum") / self.max_votes))
            else:
                return 0

    def total_votes_abs(self):
        if self.is_open:
            return self.vote_set.count()
        else:
            result = self.voting_results.aggregate(sum=Sum("vote_count"))
            if result:
                return result.get("sum")
            else:
                return 0

    def results(self):
        return self.voting_results.exclude(candidate_name="Tyhjä").order_by(
            "-vote_count"
        )

    def winners(self):
        return self.voting_results.exclude(candidate_name="Tyhjä").order_by(
            "-vote_count"
        )[: self.max_votes]

    def losers(self):
        return self.voting_results.exclude(candidate_name="Tyhjä").order_by(
            "-vote_count"
        )[self.max_votes :]

    def empty_votes(self):
        return self.voting_results.filter(candidate_name="Tyhjä")[0].vote_count

    def open_voting(self):
        self.is_open = True
        self.save()

    def close_voting(self):
        self.is_open = False
        self.is_ended = True
        self.save()

    def __str__(self):
        return self.voting_name


class NormalVoting(Voting):
    is_transferable = False


class RankedChoiceVoting(Voting):
    is_transferable = True
    voting_round = models.IntegerField(default=1)

    def grouped_results(self):
        result = []
        for i in range(1, self.round + 1):
            round_obj = {}
            round_obj["round"] = i
            round_obj["candidates"] = list(self.voting_results.filter(vote_rounds=i))
            result.append(round_obj)
        return sorted(result, key=lambda k: k["round"], reverse=True)

    def winners(self):
        result = []
        for round in self.grouped_results():
            result += list(
                map(
                    lambda y: y.candidate_name,
                    filter(lambda x: x.elected, round["candidates"]),
                )
            )
        return ", ".join(reversed(result))

    def __str__(self):
        return f"{self.voting_name} (siirtoäänivaalitapa)"


class Candidate(PolymorphicModel):
    candidate_name = models.CharField(max_length=50)

    def __str__(self):
        return self.candidate_name

class NormalCandidate(Candidate):
    voting = models.ForeignKey(
        NormalVoting, on_delete=models.CASCADE, related_name="candidates"
    )
    empty_candidate = models.BooleanField(default=False)


class RankedChoiceCandidate(Candidate):
    voting = models.ForeignKey(
        RankedChoiceVoting, on_delete=models.CASCADE, related_name="candidates"
    )


class Usertoken(models.Model):
    token = models.CharField(max_length=50, unique=True)
    alias = models.CharField(max_length=50, blank=True, unique=False)
    created = models.DateTimeField(auto_now=False, auto_now_add=True)
    activated = models.BooleanField(default=False)
    invalidated = models.BooleanField(default=False)

    def __str__(self):
        return self.token


class TokenMapping(PolymorphicModel):
    uuid = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    token = models.ForeignKey(Usertoken, on_delete=models.CASCADE)

    def get_token(self):
        return self.token

    def __str__(self):
        return str(self.uuid)

class NormalTokenMapping(TokenMapping):
    voting = models.ForeignKey(NormalVoting, on_delete=models.CASCADE)


class RankedChoiceTokenMapping(TokenMapping):
    voting = models.ForeignKey(RankedChoiceVoting, on_delete=models.CASCADE)


class RankedChoiceVoteGroup(models.Model):
    uuid = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    voting = models.ForeignKey(RankedChoiceVoting, on_delete=models.CASCADE)
    is_transferred = models.BooleanField(default=False)


class Vote(PolymorphicModel):
    uuid = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    candidate = models.ForeignKey(Candidate, on_delete=models.CASCADE)

class RankedChoiceVote(Vote):
    preference = models.IntegerField(default=0)
    voting = models.ForeignKey(RankedChoiceVoting, on_delete=models.CASCADE)
    votegroup = models.ForeignKey(RankedChoiceVoteGroup, on_delete=models.CASCADE)


class NormalVote(Vote):
    voting = models.ForeignKey(NormalVoting, on_delete=models.CASCADE)


class VotingResult(PolymorphicModel):
    candidate_name = models.CharField(max_length=50)
    vote_count = models.IntegerField(default=0)

    def vote_share(self):
        total_votes = self.voting.total_votes_abs()
        if total_votes > 0:
            return "{:.1f}".format(
                round(100 * self.vote_count / total_votes, 1)
            ).replace(".", ",")
        return "0,0"


# Voting results are freezed in this table AFTER the voting has ended.
class NormalVotingResult(VotingResult):
    voting = models.ForeignKey(
        NormalVoting, on_delete=models.CASCADE, related_name="voting_results"
    )

    def vote_share(self):
        total_votes = self.voting.total_votes_abs()
        if total_votes > 0:
            return "{:.1f}".format(
                round(100 * self.vote_count / total_votes, 1)
            ).replace(".", ",")
        return "0,0"


class RankedChoiceVotingResult(VotingResult):
    voting = models.ForeignKey(
        RankedChoiceVoting, on_delete=models.CASCADE, related_name="voting_results"
    )
    vote_rounds = models.IntegerField(default=1)
    elected = models.BooleanField(default=False)
    dropped = models.BooleanField(default=False)


class VoterStatus(PolymorphicModel):
    usertoken_token = models.CharField(max_length=50, blank=False, unique=False)
    usertoken_alias = models.CharField(max_length=50, blank=True, unique=False)
    has_voted = models.BooleanField(default=True)


class NormalVotingVoterStatus(VoterStatus):
    voting = models.ForeignKey(NormalVoting, on_delete=models.CASCADE)


class TransferableVotingVoterStatus(VoterStatus):
    voting = models.ForeignKey(RankedChoiceVoting, on_delete=models.CASCADE)
