from django.db import models
import datetime
import uuid

# Create your models here.


class Voting(models.Model):
    voting_name = models.CharField(max_length=50)
    voting_description = models.CharField(max_length=200, blank=True)
    max_votes = models.IntegerField(default=1)
    is_open = models.BooleanField(default=False)
    is_ended = models.BooleanField(default=False)

    def open_voting(self):
        self.is_open = True
        self.save()

    def close_voting(self):
        self.is_open = False
        self.is_ended = True
        self.save()

    def print_candidates(self):
        return str(candidates)

    def __str__(self):
        return self.voting_name


class Candidate(models.Model):
    voting = models.ForeignKey(Voting, on_delete=models.CASCADE)
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
        return self.uuid


class Vote(models.Model):
    uuid = models.CharField(max_length=200)
    candidate = models.ForeignKey(Candidate, on_delete=models.PROTECT)
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
