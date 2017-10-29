from django.db import models

# Create your models here.

class Voting(models.Model):
    voting_name = models.CharField(max_length=50)
    voting_description = models.CharField(max_length=200, blank=True)
    is_open = models.BooleanField(default=False)
    candidates = []

    def open_voting(self):
        self.is_open = True
        self.save()

    def close_voting(self):
        self.is_open = False
        self.save()

    def add_candidate(self, voting_name, name):
        candidate = Candidate(voting = self, candidate_name = name)
        candidate.save()
        self.candidates += candidate
        self.save()
        return candidate

    def vote(self, candidate):
        candidate.votes += 1
        candidate.save()
        return True

    def result(self):
        votes = list(map(lambda x: (x, x.votes), candidates))
        winner = max(votes)
        return winner

    def print_candidates(self):
        return str(candidates)

    def __str__(self):
        return self.voting_name


class Candidate(models.Model):
    voting = models.ForeignKey(Voting, on_delete=models.CASCADE)
    candidate_name = models.CharField(max_length=50)
    votes = models.IntegerField(default=0)
    def __str__(self):
        return self.candidate_name + " - " + voting.str()






def create_voting(name):
    voting = Voting(voting_name = name)
    voting.save()
    return voting
