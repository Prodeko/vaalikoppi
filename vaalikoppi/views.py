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

import json
from collections import Counter

from vaalikoppi.py3votecore.stv import *


def get_token_obj(request):

    session_var_name = settings.USER_TOKEN_VAR

    if session_var_name in request.session:
        cur_token = request.session[session_var_name]

        try:
            token_obj = Usertoken.objects.get(token = cur_token)
            return token_obj
        except (Usertoken.DoesNotExist):
            return None

    return None

def is_valid_token_obj(request, token_obj):

    if (token_obj is not None and token_obj.activated == True and token_obj.invalidated == False):
        return True

    return False

def is_valid_token(request):

    token_obj = get_token_obj(request)
    return is_valid_token_obj(request, token_obj)

def get_active_tokens(request):
    return Usertoken.objects.filter(activated = True, invalidated = False)

def get_eligible_active_tokens(request, voting_obj_list):

    eligible_tokens = []

    for voting_obj in voting_obj_list:
        votes_given = Vote.objects.filter(voting=voting_obj)
        active_mappings = TokenMapping.objects.filter(voting=voting_obj)
        cur_eligible_tokens = []

        for mapping in active_mappings:
            cur_token = mapping.token
            cur_votes = votes_given.filter(uuid=mapping.uuid, voting=voting_obj)

            if (is_valid_token_obj(request, cur_token) and len(cur_votes) == 0):
                cur_eligible_tokens.append(cur_token)

        eligible_tokens.append(cur_eligible_tokens)

    return zip(voting_obj_list, eligible_tokens)

# A bit double logic here, should be refactored
def is_eligible_to_vote(request, voting_obj):

    if (is_valid_token(request)):
        token_obj = get_token_obj(request)
        
        try:
            mapping = TokenMapping.objects.get(token=token_obj, voting=voting_obj)
        except (TokenMapping.DoesNotExist, TokenMapping.MultipleObjectsReturned):
            return False
        else:
            cur_votes = Vote.objects.filter(uuid=mapping.uuid, voting=voting_obj)

            # Strict policy: don't let the user vote even in a case where 0 < len(cur_votes) < max_votes. Should never happen.
            if (len(cur_votes) == 0):
                 return True

    return False

# A bit double logic here, should be refactored
def is_eligible_to_vote_transferable(request, voting_obj):
    if (is_valid_token(request)):
        token_obj = get_token_obj(request)
        try:
            mapping = TokenMappingTransferable.objects.get(token=token_obj, voting=voting_obj)
        except (TokenMappingTransferable.DoesNotExist, TokenMappingTransferable.MultipleObjectsReturned):
            return False
        else:
            cur_votes = VoteGroupTransferable.objects.filter(uuid=mapping.uuid, voting=voting_obj)
            # Strict policy: don't let the user vote even in a case where 0 < len(cur_votes) < max_votes. Should never happen.
            if (len(cur_votes) == 0):
                 return True

    print("ssssss")
    return False

def transfer_election_has_result(request, voting_obj):
    candidates = CandidateTransferable.objects.all().filter(voting = voting_obj)

    for candi in candidates:
        if (candi.has_dropped == False and candi.chosen == False):
            return False

    return True

def index(request):

    info_dict = {
        'is_valid_token' : False,
        'user_token' : 'EI KOODIA',
        'nocache_rand' : random.randint(10000, 99999)
    }

    if (is_valid_token(request)):
        info_dict['is_valid_token'] = True
        info_dict['user_token'] = get_token_obj(request).token

    return render(request, 'index.html', info_dict)

def votings(request):

    if (is_valid_token(request) == False):
        return JsonRespose('message', 'Could not return voting list due to non-eligible token.', status = 401)

    closed_regular_votings = list(Voting.objects.filter(is_open = False, is_ended = False))
    closed_transferable_votings = list(VotingTransferable.objects.filter(is_open = False, is_ended = False))
    closed_votings = sorted((closed_regular_votings + closed_transferable_votings), key=id, reverse=True)

    open_votings = []
    
    ended_regular_votings = list(Voting.objects.filter(is_open = False, is_ended = True))
    ended_transferable_votings = list(VotingTransferable.objects.filter(is_open = False, is_ended = True))
    ended_votings = sorted((ended_regular_votings + ended_transferable_votings), key=id, reverse=True)
    
    for voting in Voting.objects.filter(is_open = True, is_ended = False):
        if (is_eligible_to_vote(request, voting)):
            open_votings.append(voting)
        else:
            closed_votings.insert(0, voting)
    for voting in VotingTransferable.objects.filter(is_open = True, is_ended = False):
        if (is_eligible_to_vote_transferable(request, voting)):
            open_votings.append(voting)
        else:
            closed_votings.insert(0, voting)

    return render(request, 'votings.html', {
        'closed_votings': closed_votings,
        'open_votings': open_votings,
        'ended_votings': ended_votings,
    })
    

@csrf_exempt
def vote(request, voting_id):

    if (is_eligible_to_vote(request, voting_id) == False):
        return JsonResponse({'message':'not allowed to vote in this voting!'}, status=403)

    voting_obj = get_object_or_404(Voting, pk=voting_id)
    token_obj = get_token_obj(request)

    candidates = []
    candidates_noempty = []
    candidate_objs = []
    empty_candidate = Candidate.objects.get(voting=voting_obj, empty_candidate=True)

    if request.POST.getlist('candidates[]'):
        candidates = request.POST.getlist('candidates[]')
    else:
        return JsonResponse({'message':'candidates not provided'}, status=400)

    candidates_noempty = [x for x in candidates if x != empty_candidate.id]

    if (len(candidates_noempty) != len(set(candidates_noempty))):
        return JsonResponse({'message':'multiple votes for same candidate'}, status=400)

    empty_votes = voting_obj.max_votes - len(candidates_noempty)

    for candi_id in candidates_noempty:

        try:
            candidate_obj = Candidate.objects.get(pk = candi_id, voting = voting_obj)
            candidate_objs.append(candidate_obj)
        except (Candidate.DoesNotExist, Candidate.MultipleObjectsReturned):
            return JsonResponse({'message':'no such candidate for this voting'}, status=400)

    for i in range(0, empty_votes):
        candidate_objs.append(empty_candidate)

    try:
        mapping = TokenMapping.objects.get(token=token_obj, voting=voting_obj)
    except (TokenMapping.DoesNotExist):
        return JsonResponse({'message':'no uuid for token'}, status=403)

    # Double-check...
    cur_votes = Vote.objects.all().filter(uuid=mapping.uuid, voting=voting_obj)
    if len(cur_votes) != 0:
         return JsonResponse({'message':'already voted in this voting!'}, status=403)

    for candidate_obj in candidate_objs:
        Vote(uuid=mapping.uuid, candidate=candidate_obj, voting=voting_obj).save()

    return votings(request)

@csrf_exempt
def vote_transferable(request, voting_id):
    ## NEED TO CHECK THAT POSTS CORRECTLY
    if (is_eligible_to_vote_transferable(request, voting_id) == False):
        return JsonResponse({'message':'not allowed to vote in this voting!'}, status=403)

    voting_obj = get_object_or_404(VotingTransferable, pk=voting_id)
    token_obj = get_token_obj(request)

    # check these
    candidates = []
    candidate_objs = {}
    vote_objs = []
    votes_noempty = []
    votes = []
   # empty_candidate = Candidate.objects.get(voting=voting_obj, empty_candidate=True)

    print(request.POST.getlist('candidates[]'))
    if request.POST.getlist('candidates[]'):
        candidates = request.POST.getlist('candidates[]')
    else:
        return JsonResponse({'message':'candidates not provided'}, status=400)

    # Candi is pair of id:order
    for candi in candidates:
        try:
            print(candi.split(':')[0])
            candidate_obj = CandidateTransferable.objects.get(pk = candi.split(':')[0], voting = voting_obj)
            if candi.split(':')[1] != '-':
                candidate_objs[candi.split(':')[1]] = candidate_obj
        except (CandidateTransferable.DoesNotExist, CandidateTransferable.MultipleObjectsReturned):
            return JsonResponse({'message':'no such candidate for this voting'}, status=400)

    print("c")
    try:
        mapping = TokenMappingTransferable.objects.get(token=token_obj, voting=voting_obj)
    except (TokenMappingTransferable.DoesNotExist):
        return JsonResponse({'message':'no uuid for token'}, status=403)

    # Double-check..
    
    # !!!!!!!!!! VERY IMPORTANT TODO!!!!!!!!

    cur_votes = VoteGroupTransferable.objects.all().filter(uuid=mapping.uuid, voting=voting_obj)
    if len(cur_votes) != 0:
         return JsonResponse({'message':'already voted in this voting!'}, status=403)

    # Create Vote group
    vote_group = VoteGroupTransferable(uuid=mapping.uuid, voting=voting_obj, is_transferred=False)
    vote_group.save()
    print(vote_group)
    
    for key in candidate_objs:
        VoteTransferable(uuid=mapping.uuid, candidate=candidate_objs[key], voting=voting_obj, preference=key, votegroup=vote_group).save()

    return votings(request)

@csrf_exempt
def user_status(request):

    if (is_valid_token(request) == True):
        return JsonResponse({'status' : 1, 'token': get_token_obj(request).token, 'message':'Token is active and valid.'}, status = 200)

    return JsonResponse({'status' : 0, 'token' : '', 'message': 'Token does not exist, is not active or has been invalidated.'}, status = 200)

@csrf_exempt
def user_login(request):

    session_var_name = settings.USER_TOKEN_VAR

    if request.POST.get('token'):
        token = request.POST.get('token')
    else:
        return JsonResponse({'message':'token not provided'}, status=400)

    token_obj = get_object_or_404(Usertoken, token=token)

    if token_obj.activated == True and token_obj.invalidated == False:
        request.session[session_var_name] = token_obj.token
        return JsonResponse({'message':'login success', 'token': token_obj.token}, status=200)

    return JsonResponse({'message':'invalid token'}, status=403)

@csrf_exempt
def user_logout(request):

    session_var_name = settings.USER_TOKEN_VAR
    request.session[session_var_name] = ''
    request.session.flush()

    if (is_valid_token(request) == False):
        return JsonResponse({'message':'logged out', 'status': 0}, status=200)

    return JsonResponse({'message':'could not log out'}, status=500)


@login_required
def voting_results(request):

    votings = VotingResult.objects.all()

    return render(request, 'voting_results.html', {
        'votings': votings,
    })

@login_required
def admin_tokens(request):

    all_tokens = Usertoken.objects.all()
    new_tokens = Usertoken.objects.filter(activated = False, invalidated = False).count()
    active_tokens = get_active_tokens(request).count()
    invalid_tokens = Usertoken.objects.filter(invalidated = True).count()

    return render(request, 'admin-tokens.html', {
        'tokens': all_tokens,
        'new_tokens': new_tokens,
        'active_tokens': active_tokens,
        'invalid_tokens': invalid_tokens,
    })

def tokens(request):
    tokens = Usertoken.objects.filter(activated = False, invalidated = False)
    return render(request, 'tokens.html', {
        'tokens': tokens,
    })

@login_required
def admin_votings(request):
    return render(request, 'admin-votings.html')

@csrf_exempt
@login_required
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
@login_required
def invalidate_token(request):

    if request.POST.get('token'):
        token = request.POST.get('token')
    else:
        return JsonResponse({'message':'token not provided'}, status=400)

    token_obj = get_object_or_404(Usertoken, token=token)

    token_obj.invalidated = True
    token_obj.save()

    return JsonResponse({'message':'success'}, status=200)

@csrf_exempt
@login_required
def activate_token(request):

    if request.POST.get('token'):
        token = request.POST.get('token')
    else:
        return JsonResponse({'message':'token not provided'}, status=400)

    token_obj = get_object_or_404(Usertoken, token=token)

    token_obj.activated = True
    token_obj.save()

    return JsonResponse({'message':'success'}, status=200)

@csrf_exempt
@login_required
def invalidate_all_tokens(request):

    Usertoken.objects.all().filter(activated = True).update(invalidated = True)

    return JsonResponse({'message':'success'}, status=200)

@csrf_exempt
@login_required
def create_voting(request):
    is_transferable = (request.POST.get('is_transferable') == "true")
    voting_name = request.POST.get('voting_name')
    voting_description = request.POST.get('voting_description')
    max_votes = int(request.POST.get('max_votes'))

    if is_transferable:
        voting_obj = VotingTransferable(voting_name=voting_name, voting_description=voting_description)
        voting_obj.save()
    else:
        voting_obj = Voting(voting_name=voting_name, voting_description=voting_description, max_votes=max_votes)
        voting_obj.save()
    return JsonResponse({'message': is_transferable }, status=200)

@csrf_exempt
@login_required
def add_candidate(request, voting_id):
    if (request.POST.get('is_transferable') == "true"):
        voting = get_object_or_404(VotingTransferable, pk=voting_id)
        candidate_name = request.POST.get('candidate_name')
        candidate = CandidateTransferable(voting=voting, candidate_name=candidate_name)
        candidate.save()
    else: 
        voting = get_object_or_404(Voting, pk=voting_id)
        candidate_name = request.POST.get('candidate_name')
        candidate = Candidate(voting=voting, candidate_name=candidate_name)
        candidate.save()
    return JsonResponse({'message':'success'}, status=200)

@csrf_exempt
@login_required
def remove_candidate(request, candidate_id):
    if (request.POST.get('is_transferable') == "true"):
        get_object_or_404(CandidateTransferable, pk=candidate_id).delete()
    else:
        get_object_or_404(Candidate, pk=candidate_id).delete()
    return JsonResponse({'message':'success'}, status=200)


@csrf_exempt
@login_required
def open_voting(request, voting_id):
    if (request.POST.get('is_transferable') == "true"):
        voting_obj = get_object_or_404(VotingTransferable, pk=voting_id)
    else:
        voting_obj = get_object_or_404(Voting, pk=voting_id)
        Candidate(candidate_name='Tyhjä', empty_candidate=True, voting=voting_obj).save()

    if voting_obj.is_open or voting_obj.is_ended:
        return JsonResponse({'message':'Voting is already open or has ended'}, status=403)

    active_tokens = get_active_tokens(request)

    if (request.POST.get('is_transferable') == "true"):
        for cur_token in active_tokens:
            TokenMappingTransferable(token=cur_token, voting=voting_obj).save()      
    else:
        for cur_token in active_tokens:
            TokenMapping(token=cur_token, voting=voting_obj).save()

    voting_obj.open_voting()
    return JsonResponse({'message':'voting opened'}, status=200)

# TODO checked

@csrf_exempt
@login_required
def close_voting_transferable(request, voting_id):

    voting_obj = get_object_or_404(Voting, pk=voting_id)
    not_voted_tokens = []

    if voting_obj.is_open == False or voting_obj.is_ended == True:
        return JsonResponse({'message':'voting is not open or has ended'}, status=403)

    for mapping in TokenMapping.objects.all().filter(voting=voting_obj):
        cur_votes = Vote.objects.all().filter(uuid=mapping.uuid, voting=voting_obj)
        if len(cur_votes) > voting_obj.max_votes:
            return JsonResponse({'message':'Security compromised - too many votes from a single voter'}, status=500)
        if (len(cur_votes) == 0):
            not_voted_tokens.append(mapping.get_token().token)

    voting_obj.close_voting()

    quota = len(TokenMapping.objects.all().filter(voting=voting_obj))/(voting_obj.max_votes + 1) + 1

    TokenMapping.objects.all().filter(voting=voting_obj).delete()

    while not transfer_election_has_result(request, voting_obj):
        #to do implement iterative LookupError
        continue

@csrf_exempt
@login_required
def close_voting(request, voting_id):
    is_transferable = (request.POST.get('is_transferable') == "true")

    if is_transferable:
        voting_obj = get_object_or_404(VotingTransferable, pk=voting_id)
    else:
        voting_obj = get_object_or_404(Voting, pk=voting_id)

    not_voted_tokens = []
    
    if not voting_obj.is_open or voting_obj.is_ended:
        return JsonResponse({'message':'Voting is not open or has ended'}, status=403)

    if is_transferable:
        mappings = TokenMappingTransferable.objects.all().filter(voting=voting_obj)
    else:
        mappings = TokenMapping.objects.all().filter(voting=voting_obj)

    for mapping in mappings:
        if is_transferable:
            cur_votes = VoteTransferable.objects.all().filter(uuid=mapping.uuid, voting=voting_obj)
        else:
            cur_votes = Vote.objects.all().filter(uuid=mapping.uuid, voting=voting_obj)
            if len(cur_votes) > voting_obj.max_votes:
                return JsonResponse({'message':'Security compromised - too many votes from a single voter'}, status=500)
            if (len(cur_votes) == 0):
                not_voted_tokens.append(mapping.get_token().token)
        
        mapping.delete()

    voting_obj.close_voting()

    results = calculate_results_stv(request, voting_obj)

    for cur_candidate in voting_obj.candidates.all():
        if is_transferable:
            cur_vote_count = len(VoteTransferable.objects.all().filter(voting = voting_obj, candidate = cur_candidate))
            winners = results['rounds'][0]['winners']
            for winner in winners:
                winner_object = CandidateTransferable.objects.get(pk=winner)
                winner_object.chosen = True
                winner_object.save()
            VotingResultTransferable(voting = voting_obj, candidate_name = cur_candidate.candidate_name, vote_count = cur_vote_count).save()
        else:
            cur_vote_count = len(Vote.objects.all().filter(voting = voting_obj, candidate = cur_candidate))
            VotingResult(voting = voting_obj, candidate_name = cur_candidate.candidate_name, vote_count = cur_vote_count).save()


    if is_transferable:
        for round in results['rounds']:
            print(round)


    return JsonResponse({'message':'voting closed', 'not_voted_tokens':not_voted_tokens}, status=200)


def calculate_results_stv(request, voting_obj):
    inputs = calculate_stv(request, voting_obj.id)
    results = STV(inputs, required_winners=1).as_dict()
    
    return results


def calculate_stv(request, voting_id):
    ballots = []
    ballots2 = []
    countdict = {}
    keysdict = {}

    voting_obj = get_object_or_404(VotingTransferable, pk=voting_id)
    votegroups = VoteGroupTransferable.objects.all().filter(voting = voting_obj)
    for vote_group in votegroups:
        votes = VoteTransferable.objects.all().filter(votegroup = vote_group).order_by('preference')
        vote_array = []
        keystring = ""
        for vote in votes:
            keystring += str(vote.candidate) + "-" + str(vote.preference)
            vote_array.append(str(vote.candidate.id))
        if keystring in countdict:
            countdict[keystring] = countdict[keystring]+1
        else:
            keysdict[keystring] = vote_array
            countdict[keystring] = 1


    for key, value in keysdict.items():
        ballots.append({"count": countdict[key], "ballot": value})
    #print(ballots)
    
    return ballots


def test(request):
    inputs = calculate_stv(request, 7)
    ballots = [
            {"count": 56, "ballot": ["c1", "c2", "c3"]},
            {"count": 40, "ballot": ["c2", "c3", "c1"]},
            {"count": 20, "ballot": ["c3", "c1", "c2"]}
            ]
    results = STV(ballots, required_winners=1).as_dict()
    #print(results['rounds']) 
    #asd = results['rounds'][0]['winners']
    for round in results['rounds']:
        print(round)
        for d in round['tallies']:
            print(d)
    print(results['rounds']) 
    return render_to_response("test_stvresults.html", {'data': results})

@csrf_exempt
@login_required
def admin_voting_list(request):
    # TODO: Votings are sorted by ID even though both voting types have separate running IDs
    closed_regular_votings = list(Voting.objects.filter(is_open = False, is_ended = False))
    closed_transferable_votings = list(VotingTransferable.objects.filter(is_open = False, is_ended = False))
    closed_votings = sorted((closed_regular_votings + closed_transferable_votings), key=id, reverse=True)

    open_regular_votings = list(Voting.objects.filter(is_open = True, is_ended = False))
    open_transferable_votings = list(VotingTransferable.objects.filter(is_open = True, is_ended = False))
    open_votings = sorted((open_regular_votings + open_transferable_votings), key=id, reverse=True)
    
    ended_regular_votings = list(Voting.objects.filter(is_open = False, is_ended = True))
    ended_transferable_votings = list(VotingTransferable.objects.filter(is_open = False, is_ended = True))
    ended_votings = sorted((ended_regular_votings + ended_transferable_votings), key=id, reverse=True)
    
    active_tokens_count = len(get_active_tokens(request))

    return render(request, 'admin-voting-list.html', {
        'closed_votings': closed_votings,
        'open_votings': open_votings,
        'ended_votings': ended_votings,
        'active_tokens_count': active_tokens_count
    })