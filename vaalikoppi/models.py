import math
import uuid

from django.db import models
from django.db.models import Sum


class Voting(models.Model):
    voting_name = models.CharField(max_length=50)
    voting_description = models.CharField(max_length=200, blank=True)
    max_votes = models.IntegerField(default=1)
    is_open = models.BooleanField(default=False)
    is_ended = models.BooleanField(default=False)
    treshold = models.FloatField(default=500.0)
    is_transferable = False

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


class VotingTransferable(models.Model):
    voting_name = models.CharField(max_length=50)
    voting_description = models.CharField(max_length=200, blank=True)
    is_open = models.BooleanField(default=False)
    is_ended = models.BooleanField(default=False)
    round = models.IntegerField(default=1)
    is_transferable = True
    max_votes = models.IntegerField(default=1)

    def total_votes(self):
        if self.is_open:
            return int(self.votegrouptransferable_set.count())
        else:
            result = self.voting_results.aggregate(sum=Sum("vote_count"))
            if result:
                return int(math.floor(result.get("sum")))
            else:
                return 0

    def rounds_list(self):
        return range(1, self.round + 1)

    def grouped_results(self):
        result = []
        for i in self.rounds_list():
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
        return "{} (siirtoäänivaalitapa)".format(self.voting_name)


class VotingRoundTransferable(models.Model):
    voting = models.ForeignKey(Voting, on_delete=models.CASCADE)
    round = models.IntegerField(default=0)

    def total_votes(self):
        if voting.is_open:
            # TODO max_votes no more needed, change to match no. of cands
            return int(voting.vote_set.count() / self.max_votes)
        else:
            result = self.voting_results.aggregate(sum=Sum("vote_count"))
            if result:
                # TODO same changes as 4 rows upwards
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


class Candidate(models.Model):
    voting = models.ForeignKey(
        Voting, on_delete=models.CASCADE, related_name="candidates"
    )
    candidate_name = models.CharField(max_length=50)
    empty_candidate = models.BooleanField(default=False)

    def __str__(self):
        return self.candidate_name


class CandidateTransferable(models.Model):
    voting = models.ForeignKey(
        VotingTransferable, on_delete=models.CASCADE, related_name="candidates"
    )
    candidate_name = models.CharField(max_length=50)

    def __str__(self):
        return self.candidate_name


class Usertoken(models.Model):
    token = models.CharField(max_length=50, unique=True)
    created = models.DateTimeField(auto_now=False, auto_now_add=True)
    activated = models.BooleanField(default=False)
    invalidated = models.BooleanField(default=False)

    def activate(self):
        self.activated = timezone.now

    def invalidate(self):
        self.invalidated = timezone.now

    def __str__(self):
        return self.token


class TokenMapping(models.Model):
    uuid = models.UUIDField(primary_key=True, default=uuid.uuid4)
    token = models.ForeignKey(Usertoken, on_delete=models.CASCADE)
    voting = models.ForeignKey(Voting, on_delete=models.CASCADE)

    def get_uuid(self):
        return self.uuid

    def get_token(self):
        return self.token

    def __str__(self):
        return str(self.uuid)


class TokenMappingTransferable(models.Model):
    uuid = models.UUIDField(primary_key=True, default=uuid.uuid4)
    token = models.ForeignKey(Usertoken, on_delete=models.CASCADE)
    voting = models.ForeignKey(VotingTransferable, on_delete=models.CASCADE)

    def get_uuid(self):
        return self.uuid

    def get_token(self):
        return self.token

    def __str__(self):
        return str(self.uuid)


class VoteGroupTransferable(models.Model):
    uuid = models.CharField(max_length=200)
    voting = models.ForeignKey(VotingTransferable, on_delete=models.CASCADE)
    is_transferred = models.BooleanField(default=False)

    def get_uuid(self):
        return self.uuid

    def get_votes(self):
        return self.votetransferable_set


class VoteBatchTransferable(models.Model):
    voting_group = models.ForeignKey(VoteGroupTransferable, on_delete=models.CASCADE)
    candi = models.ForeignKey(CandidateTransferable, on_delete=models.CASCADE)
    vote_count = models.FloatField(default=0.0)

    def get_uuid(self):
        return self.uuid

    def get_votes(self):
        return self.votetransferable_set


class VoteTransferable(models.Model):
    uuid = models.CharField(max_length=200)
    candidate = models.ForeignKey(CandidateTransferable, on_delete=models.CASCADE)
    preference = models.IntegerField(default=0)
    voting = models.ForeignKey(VotingTransferable, on_delete=models.CASCADE)
    votegroup = models.ForeignKey(VoteGroupTransferable, on_delete=models.CASCADE)

    def get_uuid(self):
        return self.uuid

    def get_candidate(self):
        return self.candidate

    def get_votegroup(self):
        return self.votegroup


class Vote(models.Model):
    uuid = models.CharField(max_length=200)
    candidate = models.ForeignKey(Candidate, on_delete=models.CASCADE)
    voting = models.ForeignKey(Voting, on_delete=models.CASCADE)

    def get_uuid(self):
        return self.uuid

    def get_candidate(self):
        return self.candidate


# Voting results are freezed in this table AFTER the voting has ended.
class VotingResult(models.Model):
    voting = models.ForeignKey(
        Voting, on_delete=models.CASCADE, related_name="voting_results"
    )
    candidate_name = models.CharField(max_length=50)
    vote_count = models.IntegerField(default=0)

    def vote_share(self):
        total_votes = self.voting.total_votes_abs()
        if total_votes > 0:
            return "{:.1f}".format(
                round(100 * self.vote_count / total_votes, 1)
            ).replace(".", ",")
        return "0,0"


class VotingResultTransferable(models.Model):
    voting = models.ForeignKey(
        VotingTransferable, on_delete=models.CASCADE, related_name="voting_results"
    )
    candidate_name = models.CharField(max_length=50)
    vote_count = models.FloatField(default=0.0)
    vote_rounds = models.IntegerField(default=1)
    elected = models.BooleanField(default=False)
    dropped = models.BooleanField(default=False)
