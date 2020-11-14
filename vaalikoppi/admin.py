from django.contrib import admin

from .models import *

admin.site.register(Usertoken)
admin.site.register(NormalVoting)
admin.site.register(NormalCandidate)
admin.site.register(NormalVote)
admin.site.register(NormalTokenMapping)
admin.site.register(NormalVotingResult)
admin.site.register(NormalVotingVoterStatus)

admin.site.register(RankedChoiceVoting)
admin.site.register(RankedChoiceCandidate)
admin.site.register(RankedChoiceVoteGroup)
admin.site.register(RankedChoiceVote)
admin.site.register(RankedChoiceTokenMapping)
admin.site.register(RankedChoiceVotingResult)
admin.site.register(RankedChoiceVotingVoterStatus)
