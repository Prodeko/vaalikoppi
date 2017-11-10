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


#@login_required(login_url='/login/')
# Ei toiminu ilman tätä, pitäis tutkia
@csrf_exempt
def vote(request, voting_id):
    voting = get_object_or_404(Voting, pk=voting_id)
    try:
        selected_candidate = voting.candidate_set.get(pk=request.POST['candidate'])
    except (KeyError, Candidate.DoesNotExist):
        # Redisplay the question voting form.
        return JsonResponse({'message':'candidate does not exist'}, status=400)
    else:
        selected_candidate.vote()
		# Ei jostain syystä toiminu redirect('votings'), pitäis tutkia
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