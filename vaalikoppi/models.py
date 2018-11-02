from django.db import models
import datetime
import uuid
from django.db.models import Count, Min, Sum, Avg
import math

# Create your models here.


class Voting(models.Model):
    voting_name = models.CharField(max_length=50)
    voting_description = models.CharField(max_length=200, blank=True)
    max_votes = models.IntegerField(default=1)
    is_open = models.BooleanField(default=False)
    is_ended = models.BooleanField(default=False)

    def total_votes(self):
        if self.is_open:
            return int(math.floor(self.vote_set.count() / self.max_votes))
        else:
            result = self.votingresult_set.aggregate(sum=Sum('vote_count'))
            if result:
                return int(math.floor(result.get('sum') / self.max_votes))
            else:
                return 0
    
    def total_votes_abs(self):
        if self.is_open:
            return self.vote_set.count()
        else:
            result = self.votingresult_set.aggregate(sum=Sum('vote_count'))
            if result:
                return result.get('sum')
            else:
                return 0
                
    def results(self):
        return self.votingresult_set.exclude(candidate_name = 'Tyhjä').order_by('-vote_count')

    def winners(self):
        return self.votingresult_set.exclude(candidate_name = 'Tyhjä').order_by('-vote_count')[:self.max_votes]

    def losers(self):
        return self.votingresult_set.exclude(candidate_name = 'Tyhjä').order_by('-vote_count')[self.max_votes:]

    def empty_votes(self):
        return self.votingresult_set.filter(candidate_name = 'Tyhjä')[0].vote_count

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
    voting = models.ForeignKey(Voting, on_delete=models.CASCADE)
    candidate_name = models.CharField(max_length=50)
    empty_candidate = models.BooleanField(default=False)

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
        return self.uuid


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
    voting = models.ForeignKey(Voting, on_delete=models.CASCADE)
    candidate_name = models.CharField(max_length=50)
    vote_count = models.IntegerField(default=0)
    
    def vote_share(self):
        total_votes = self.voting.total_votes_abs()
        if (total_votes > 0):
            return "{:.1f}".format(round(100 * self.vote_count / total_votes, 1)).replace('.', ',')
        return "0,0"
