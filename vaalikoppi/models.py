import math
from uuid import uuid4

from django.db import models
from django.db.models import Sum


class Voting(models.Model):
    voting_name = models.CharField(max_length=50)
    voting_description = models.CharField(max_length=200, blank=True)
    max_votes = models.IntegerField(default=1)
    is_open = models.BooleanField(default=False)
    is_ended = models.BooleanField(default=False)
    treshold = models.FloatField(default=500.0)
    is_password_protected = models.BooleanField(default=False)
    voting_password = models.CharField(max_length=50, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)

    # For the purpose of getting unique DOM element IDs
    def pseudo_unique_id(self):
        if self.is_ranked_choice:
            return f"{self.id}-ranked-choice"
        return f"{self.id}-normal"

    def total_votes_abs(self):
        if self.is_open:
            return self.votes.count()
        else:
            result = self.voting_results.aggregate(sum=Sum("vote_count"))
            if result:
                return result.get("sum")
            else:
                return 0

    def grouped_results(self):
        res_all = list(self.voting_results.all())
        res_empty = [r for r in res_all if r.candidate_name == "Tyhjä"]
        res_non_empty = [r for r in res_all if r.candidate_name != "Tyhjä"]

        return res_empty, res_non_empty

    def results(self):
        _, res_non_empty = self.grouped_results()
        return res_non_empty.sort(key=lambda x: -x.vote_count)

    def winners(self):
        _, res_non_empty = self.grouped_results()
        return res_non_empty.sort(key=lambda x: -x.vote_count)[: self.max_votes]

    def losers(self):
        _, res_non_empty = self.grouped_results()
        return res_non_empty.sort(key=lambda x: -x.vote_count)[self.max_votes :]

    def empty_votes(self):
        res_empty, _ = self.grouped_results()
        return res_empty[0].vote_count

    def open_voting(self):
        self.is_open = True
        self.save()

    def close_voting(self):
        self.is_open = False
        self.is_ended = True
        self.save()

    class Meta:
        abstract = True

    def __str__(self):
        return self.voting_name


class NormalVoting(Voting):
    is_ranked_choice = False

    def total_votes(self):
        if self.is_open:
            return int(math.floor(self.votes.count() / self.max_votes))
        else:
            result = self.voting_results.aggregate(sum=Sum("vote_count"))
            if result:
                return int(math.floor(result.get("sum") / self.max_votes))
            else:
                return 0


class RankedChoiceVoting(Voting):
    is_ranked_choice = True
    voting_round = models.IntegerField(default=1)

    def total_votes(self):
        if self.is_open:
            return int(self.votegroups.count())
        else:
            result = self.voting_results.aggregate(sum=Sum("vote_count"))
            if result:
                return int(math.floor(result.get("sum")))
            else:
                return 0

    def grouped_results(self):
        result = []
        voting_results = list(self.voting_results.all())
        for i in range(1, self.voting_round + 1):
            round_obj = {}
            round_obj["round"] = i
            round_obj["candidates"] = [r for r in voting_results if r.vote_rounds == i]
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


class Candidate(models.Model):
    candidate_name = models.CharField(max_length=50)

    class Meta:
        abstract = True

    def __str__(self):
        return self.candidate_name


class NormalCandidate(Candidate):
    voting = models.ForeignKey(
        NormalVoting, on_delete=models.CASCADE, related_name="candidates"
    )
    empty_candidate = models.BooleanField(default=False)


class RankedChoiceCandidate(Candidate):
    voting = models.ForeignKey(
        RankedChoiceVoting, on_delete=models.CASCADE, related_name="candidates",
    )


class Usertoken(models.Model):
    token = models.CharField(max_length=50, unique=True)
    alias = models.CharField(max_length=50, blank=True, unique=False)
    created = models.DateTimeField(auto_now=False, auto_now_add=True)
    activated = models.BooleanField(default=False)
    invalidated = models.BooleanField(default=False)

    def __str__(self):
        return self.token


class TokenMapping(models.Model):
    uuid = models.UUIDField(primary_key=True, default=uuid4, editable=False)
    token = models.ForeignKey(
        Usertoken, on_delete=models.CASCADE, related_name="%(class)s_token"
    )

    def get_token(self):
        return self.token

    class Meta:
        abstract = True

    def __str__(self):
        return str(self.uuid)


class NormalTokenMapping(TokenMapping):
    voting = models.ForeignKey(
        NormalVoting, on_delete=models.CASCADE, related_name="token_mappings"
    )


class RankedChoiceTokenMapping(TokenMapping):
    voting = models.ForeignKey(
        RankedChoiceVoting, on_delete=models.CASCADE, related_name="token_mappings"
    )


class RankedChoiceVoteGroup(models.Model):
    uuid = models.UUIDField(default=uuid4, editable=False)
    voting = models.ForeignKey(
        RankedChoiceVoting, on_delete=models.CASCADE, related_name="votegroups"
    )
    is_transferred = models.BooleanField(default=False)


class Vote(models.Model):
    uuid = models.UUIDField(default=uuid4, editable=False)

    class Meta:
        abstract = True


class RankedChoiceVote(Vote):
    preference = models.IntegerField(default=0)
    voting = models.ForeignKey(
        RankedChoiceVoting, on_delete=models.CASCADE, related_name="votes"
    )
    votegroup = models.ForeignKey(RankedChoiceVoteGroup, on_delete=models.CASCADE)
    candidate = models.ForeignKey(RankedChoiceCandidate, on_delete=models.CASCADE)


class NormalVote(Vote):
    voting = models.ForeignKey(
        NormalVoting, on_delete=models.CASCADE, related_name="votes"
    )
    candidate = models.ForeignKey(NormalCandidate, on_delete=models.CASCADE)


class VotingResult(models.Model):
    candidate_name = models.CharField(max_length=50)
    vote_count = models.IntegerField(default=0)

    class Meta:
        abstract = True


# Voting results are freezed in this table AFTER the voting has ended.
class NormalVotingResult(VotingResult):
    voting = models.ForeignKey(
        NormalVoting, on_delete=models.CASCADE, related_name="voting_results"
    )
    vote_share = models.FloatField(default=0.0)


class RankedChoiceVotingResult(VotingResult):
    voting = models.ForeignKey(
        RankedChoiceVoting, on_delete=models.CASCADE, related_name="voting_results"
    )
    vote_rounds = models.IntegerField(default=1)
    elected = models.BooleanField(default=False)
    dropped = models.BooleanField(default=False)


class VoterStatus(models.Model):
    usertoken_token = models.CharField(max_length=50, blank=False, unique=False)
    usertoken_alias = models.CharField(max_length=50, blank=True, unique=False)
    has_voted = models.BooleanField(default=True)

    class Meta:
        abstract = True


class NormalVotingVoterStatus(VoterStatus):
    voting = models.ForeignKey(
        NormalVoting, on_delete=models.CASCADE, related_name="voter_statuses"
    )

    class Meta:
        verbose_name_plural = "Normal voting voter statuses"


class RankedChoiceVotingVoterStatus(VoterStatus):
    voting = models.ForeignKey(
        RankedChoiceVoting, on_delete=models.CASCADE, related_name="voter_statuses"
    )

    class Meta:
        verbose_name_plural = "Ranked choice voting voter statuses"
