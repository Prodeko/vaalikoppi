from django.contrib import admin

from .models import Voting, Candidate


admin.site.register(Voting)
admin.site.register(Candidate)
