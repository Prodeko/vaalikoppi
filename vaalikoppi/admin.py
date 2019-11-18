from django.contrib import admin

from .models import *

admin.site.register(Voting)
admin.site.register(Candidate)
admin.site.register(Usertoken)
admin.site.register(TokenMapping)
admin.site.register(Vote)
admin.site.register(VotingResult)

admin.site.register(VotingTransferable)
admin.site.register(CandidateTransferable)
admin.site.register(VoteGroupTransferable)
admin.site.register(VoteTransferable)
admin.site.register(TokenMappingTransferable)
admin.site.register(VotingResultTransferable)