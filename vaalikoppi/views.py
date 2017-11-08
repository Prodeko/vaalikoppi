from django.shortcuts import render, render_to_response, redirect, HttpResponse, HttpResponseRedirect
from django.http import HttpResponseNotFound, HttpResponseForbidden, StreamingHttpResponse, JsonResponse
from django.template import RequestContext
from django.shortcuts import get_object_or_404
from django.contrib.auth.decorators import login_required
from django.contrib.auth import update_session_auth_hash
from django.contrib.admin.views.decorators import staff_member_required
from django.contrib import messages
from django.shortcuts import render
from django.db.models import Q, Max
from .models import Voting, Candidate
from django.views.decorators.csrf import csrf_exempt


# Create your views here.
from django.http import HttpResponse
from django.http import JsonResponse


def index(request):
    return render(request, 'index.html')

def votings(request):

    closed_votings = Voting.objects.filter(is_open = False, is_ended = False)
    open_votings = Voting.objects.filter(is_open = True, is_ended = False)
    ended_votings = Voting.objects.filter(is_open = False, is_ended = True)


    return render(request, 'votings.html', {
        'closed_votings': closed_votings,
        'open_votings': open_votings,
        'ended_votings': ended_votings,
    })

#@login_required(login_url='/login/')
# Ei toiminu ilman tätä, pitäis tutkia
@csrf_exempt
def vote(request, voting_id):
    voting = get_object_or_404(Voting, pk=voting_id)
    try:
        selected_candidate = voting.candidate_set.get(pk=request.POST['candidate'])
    except (KeyError, Candidate.DoesNotExist):
        # Redisplay the question voting form.
        return JsonResponse({'error':'candidate does not exist'},status=400)
    else:
        selected_candidate.vote()
		# Ei jostain syystä toiminu redirect('votings'), pitäis tutkia
        return JsonResponse({'status':'success'},status=200)

def get_candidates(voting_id):
    return Candidate.objects.get(voting_id)

def results(request, voting_id):
    votings = Voting.objects.filter(is_ended = True)
    return render(request, 'index.html', {
        'votings': votings
    })

'''
def result(request, voting_id):
    voting = Voting.objects.get(voting_id)
    return max(lambda x: x.votes, voting.candidate_set.all)
'''
