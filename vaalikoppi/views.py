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
from .models import *
from django.views.decorators.csrf import csrf_exempt
from django.core.files import File
import random
import os.path
from django.conf import settings
from django.db import connection

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


def admin_tokens(request):
    return render(request, 'admin-tokens.html')

def admin_votings(request):
    return render(request, 'admin-votings.html')

#@login_required(login_url='/login/')
@csrf_exempt
def vote(request, voting_id):

    session_var_name = 'cur_token'
    voting_obj = get_object_or_404(Voting, pk=voting_id)
    token_obj = Usertoken.objects.get(token = request.session[session_var_name])
	
    if request.POST.get('candidate'):
        candidate = request.POST.get('candidate')
    else:
        return JsonResponse({'message':'candidate not provided'}, status=400)

    candidate_obj = get_object_or_404(Candidate, pk=candidate)
    
    try:
        mapping = TokenMapping.objects.get(token=token_obj, voting=voting_obj)
    except (TokenMapping.DoesNotExist):
        return JsonResponse({'message':'no uuid for token'}, status=403)
    
    cur_votes = Vote.objects.all().filter(uuid=mapping.uuid, voting=voting_obj)
    if len(cur_votes) >= voting_obj.max_votes:
         return JsonResponse({'message':'too many votes for this voting'}, status=403)
		 
    Vote(uuid=mapping.uuid, candidate=candidate_obj, voting=voting_obj).save()
	
    return JsonResponse({'message':'success'}, status=200)

def get_candidates(voting_id):
    return Candidate.objects.get(voting_id)

def results(request, voting_id):
    votings = Voting.objects.filter(is_ended = True)
    return render(request, 'index.html', {
        'votings': votings
    })

@csrf_exempt
def generate_tokens(request):

    count = 1
	
    if request.POST.get('count'):
        count = int(request.POST.get('count'))

    with open(os.path.join(settings.BASE_DIR, 'wordlist.txt')) as f:
        words = [x.strip() for x in f] 
    
    random_gen = random.SystemRandom()
    word_count = 4
	
    for i in range(0, count):

        separator_int = random_gen.randint(0,9)
        cur_token = str(separator_int).join(random_gen.sample(words, word_count))
        Usertoken(token=cur_token).save()
    
    return JsonResponse({'message':'success'}, status=200)
	

@csrf_exempt
def invalidate_token(request):

    if request.POST.get('token'):
        token = request.POST.get('token')
    else:
        return JsonResponse({'message':'token not provided'}, status=400)

    try:
        token_obj = Usertoken.objects.get(token = token)
    except (Usertoken.DoesNotExist):
        return JsonResponse({'message':'token does not exist'}, status=404)

    token_obj.invalidated = True
    token_obj.save()

    return JsonResponse({'message':'success'}, status=200)
	
	
@csrf_exempt
def user_status(request):

    session_var_name = 'cur_token'

    if session_var_name in request.session:
        cur_token = request.session[session_var_name]

        try:
            token_obj = Usertoken.objects.get(token = cur_token)
        except (Usertoken.DoesNotExist):
            return JsonResponse({'status':0, 'message':'token does not exist'}, status=200)
        
        return JsonResponse({'status':1, 'token':cur_token, 'activated':token_obj.activated, 'invalidated':token_obj.invalidated, 'message':'token found'}, status=200)

    else:
        return JsonResponse({'status':0, 'message':'token does not exist'}, status=200)
        
@csrf_exempt
def user_login(request):

    session_var_name = 'cur_token'

    if request.POST.get('token'):
        token = request.POST.get('token')
    else:
        return JsonResponse({'message':'token not provided'}, status=400)

    try:
        token_obj = Usertoken.objects.get(token = token)
    except (Usertoken.DoesNotExist):
        return JsonResponse({'message':'token does not exist'}, status=401)

    if token_obj.invalidated == False:
        token_obj.activated = True
        token_obj.save()
        request.session[session_var_name] = token_obj.token
		
        return JsonResponse({'message':'login success', 'token': token_obj.token}, status=200)
    
    return JsonResponse({'message':'invalid token'}, status=403)
	
@csrf_exempt
def open_voting(request, voting_id):

    voting_obj = get_object_or_404(Voting, pk=voting_id)

    if voting_obj.is_open == True or voting_obj.is_ended == True:
        return JsonResponse({'message':'voting is open or has ended'}, status=403)

    active_tokens = Usertoken.objects.all().filter(activated=True, invalidated=False)
    
    for cur_token in active_tokens:
        TokenMapping(token=cur_token, voting=voting_obj).save()
    
    voting_obj.open_voting()
    return JsonResponse({'message':'voting opened'}, status=200)
	
@csrf_exempt
def close_voting(request, voting_id):

    voting_obj = get_object_or_404(Voting, pk=voting_id)

    if voting_obj.is_open == False or voting_obj.is_ended == True:
        return JsonResponse({'message':'voting is not open or has ended'}, status=403)
    
    for mapping in TokenMapping.objects.all().filter(voting=voting_obj):
        cur_votes = Vote.objects.all().filter(uuid=mapping.uuid, voting=voting_obj)
        if len(cur_votes) > voting_obj.max_votes:
            return JsonResponse({'message':'security compromised - too many votes from a single voter'}, status=500)

    voting_obj.close_voting()
    TokenMapping.objects.all().filter(voting=voting_obj).delete()
	
    return JsonResponse({'message':'voting opened'}, status=200)
	
@csrf_exempt
def admin_voting_list(request):

    closed_votings = Voting.objects.filter(is_open = False, is_ended = False)
    open_votings = Voting.objects.filter(is_open = True, is_ended = False)
    ended_votings = Voting.objects.filter(is_open = False, is_ended = True)
	
    return render(request, 'admin-voting-list.html', {
        'closed_votings': closed_votings,
        'open_votings': open_votings,
        'ended_votings': ended_votings,
    })