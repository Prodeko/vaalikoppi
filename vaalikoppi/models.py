from django.db import models

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

    def add_candidate(self, candidate):
        candidates += candidate
        self.save()

    def result(self):
        winner = max(lambda x: x.votes, candidates)
        return winner.candidate_name

    def print_candidates(self):
        return str(candidates)

    def vote(self, candidate):
        candidate.votes += 1
        candidate.save()

    def __str__(self):
        return self.voting_name


class Candidate(models.Model):
    voting = models.ForeignKey(Voting, on_delete=models.CASCADE)
    candidate_name = models.CharField(max_length=50)
    votes = models.IntegerField(default=0)

    def __str__(self):
        return self.candidate_name

def get_candidates(voting_id):
    return Candidate.objects.get(voting_id)
