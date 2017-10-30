from django.shortcuts import render, render_to_response, redirect, HttpResponse, HttpResponseRedirect
from django.http import HttpResponseNotFound, HttpResponseForbidden, StreamingHttpResponse, JsonResponse
from django.template import RequestContext
from django.shortcuts import get_object_or_404
from django.contrib.auth.decorators import login_required
from django.contrib.auth import update_session_auth_hash
from django.contrib.admin.views.decorators import staff_member_required
from django.contrib import messages
from django.shortcuts import render
from django.db.models import Q
from vaalikoppi.models import Voting, Candidate


# Create your views here.
from django.http import HttpResponse


def index(request):
    closed_votings = Candidate.objects.filter(voting__is_open = False, voting__is_ended = False)
    open_votings = Candidate.objects.filter(voting__is_open = True, voting__is_ended = False)
    ended_votings = Candidate.objects.filter(voting__is_open = False, voting__is_ended = True)


    return render_to_response('index.html', {
        'closed_votings': closed_votings,
        'open_votings': open_votings,
        'ended_votings': ended_votings,
    }, context_instance=RequestContext(request))


@login_required(login_url='/login/')
def vote(request, voting_id, candidate_id):
    voting = Voting.objects.get(voting_id)
    voting.vote(Candidate.objects.get(candidate_id))
    return HttpResponse("You're voting on question %s." % voting_id)
