from django.db import models
import datetime

# Create your models here.


class Voting(models.Model):
    voting_name = models.CharField(max_length=50)
    voting_description = models.CharField(max_length=200, blank=True)
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
    votes = models.IntegerField(default=0)

    def vote(self):
        self.votes += 1
        self.save()

    def __str__(self):
        return self.candidate_name

		
class Usertoken(models.Model):
    token = models.CharField(max_length=50, unique=True)
    created = models.DateTimeField(auto_now=False, auto_now_add=True)
    activated = models.DateTimeField(auto_now=False, auto_now_add=False, null=True, blank=True)
    invalidated = models.DateTimeField(auto_now=False, auto_now_add=False, null=True, blank=True)

    def activate(self):
        self.activated = timezone.now

    def invalidate(self):
        self.invalidated = timezone.now

    def __str__(self):
        return self.token
		
		
class Tokenusage(models.Model):
    token = models.ForeignKey(Usertoken, on_delete=models.CASCADE)
    voting = models.ForeignKey(Voting, on_delete=models.CASCADE)
    vote_timestamp = models.DateTimeField(auto_now=False, auto_now_add=True)
	
