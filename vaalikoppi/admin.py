from django.contrib import admin

from .models import *

admin.site.register(Voting)
admin.site.register(Candidate)
admin.site.register(Usertoken)
admin.site.register(TokenMapping)
admin.site.register(Vote)
admin.site.register(VotingResult)
admin.site.register(NormalVotingVoterStatus)

admin.site.register(RankedChoiceVoting)
admin.site.register(RankedChoiceCandidate)
admin.site.register(RankedChoiceVoteGroup)
admin.site.register(RankedChoiceVote)
admin.site.register(RankedChoiceTokenMapping)
admin.site.register(RankedChoiceVotingResult)
admin.site.register(TransferableVotingVoterStatus)
