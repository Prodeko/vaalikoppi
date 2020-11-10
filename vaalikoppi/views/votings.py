import json

from django.http import JsonResponse
from django.shortcuts import get_object_or_404, render
from django.views.decorators.http import require_http_methods
from py3votecore.stv import *
from vaalikoppi.forms import *
from vaalikoppi.models import *
from vaalikoppi.views.helpers import get_token_obj, is_valid_token


def validate_token(func):
    def wrapper(request, *args, **kwargs):
        token = get_token_obj(request)
        if is_valid_token(request):
            return func(request, token=token, *args, **kwargs)
        else:
            return render(request, "voting-list-error.html", {"token": token})

    return wrapper


# A bit double logic here, should be refactored
def is_eligible_to_vote(request, voting_obj):
    if is_valid_token(request):
        token_obj = get_token_obj(request)

        try:
            mapping = TokenMapping.objects.get(token=token_obj, voting=voting_obj)
        except Exception as e:
            return False
        else:
            cur_votes_count = Vote.objects.filter(
                uuid=mapping.uuid, voting=voting_obj
            ).count()

            # Strict policy: don't let the user vote even in a case where 0 < len(cur_votes) < max_votes. Should never happen.
            if cur_votes_count == 0:
                return True

    return False


# A bit double logic here, should be refactored
def is_eligible_to_vote_transferable(request, voting_obj):
    if is_valid_token(request):
        token_obj = get_token_obj(request)
        try:
            mapping = RankedChoiceTokenMapping.objects.get(
                token=token_obj, voting=voting_obj
            )
            cur_votes_by_token_count = (
                RankedChoiceVote.objects.all()
                .filter(uuid=mapping.uuid, voting=voting_obj)
                .count()
            )
            candidates_count = (
                RankedChoiceCandidate.objects.all().filter(voting=voting_obj).count()
            )
            if cur_votes_by_token_count == candidates_count:
                return False
        except Exception as e:
            return False
        else:
            cur_votes_count = RankedChoiceVoteGroup.objects.filter(
                uuid=mapping.uuid, voting=voting_obj
            ).count()
            # Strict policy: don't let the user vote even in a case where 0 < len(cur_votes) < max_votes. Should never happen.
            if cur_votes_count == 0:
                return True
    return False


def is_valid_voting_password(voting_password_typed, voting_obj):
    voting_password_real = voting_obj.voting_password
    voting_requires_password = voting_obj.is_password_protected

    if not voting_requires_password:
        return True

    if voting_password_typed == voting_password_real:
        return True
    return False


# Get votings list data directly
@validate_token
def votings_list_data(request, token):
    votings = Voting.objects.all()

    open_votings = votings.filter(is_open=True, is_ended=False)
    closed_votings = votings.filter(is_open=False, is_ended=False)
    ended_votings = votings.filter(is_open=False, is_ended=True)

    return {
        "is_admin": False,
        "open_votings": open_votings,
        "closed_votings": closed_votings,
        "ended_votings": ended_votings,
    }


# Get votings list rendered as html
@validate_token
def votings_list(request, token):
    return render(request, "voting-list.html", votings_list_data(request))


@validate_token
@require_http_methods(["POST"])
def vote(request, token, voting_id):
    if is_eligible_to_vote(request, voting_id) == False:
        return JsonResponse(
            {"message": "Not allowed to vote in this voting!"}, status=403
        )

    voting_obj = get_object_or_404(Voting, pk=voting_id)

    candidates = []
    candidates_noempty = []
    candidate_objs = []
    empty_candidate = Candidate.objects.get(voting=voting_obj, empty_candidate=True)

    data = json.loads(request.body.decode("utf-8"))

    ### BEGIN VOTING PASSWORD CHECK ###
    voting_password_typed = data.get("voting_password")

    if not voting_password_typed:
        voting_password_typed = ""

    if is_valid_voting_password(voting_password_typed, voting_obj) == False:
        return JsonResponse({"message": "Wrong voting password!"}, status=403)
    ### END VOTING PASSWORD CHECK ###

    candidates = data.get("candidates")

    if not candidates:
        return JsonResponse({"message": "Candidates not provided"}, status=400)

    candidates_noempty = [x for x in candidates if x != empty_candidate.id]

    if len(candidates_noempty) != len(set(candidates_noempty)):
        return JsonResponse(
            {"message": "Multiple votes for same candidate"}, status=400
        )

    empty_votes = voting_obj.max_votes - len(candidates_noempty)

    for candidate_id in candidates_noempty:
        try:
            candidate_obj = Candidate.objects.get(pk=candidate_id, voting=voting_obj)
            candidate_objs.append(candidate_obj)
        except (Candidate.DoesNotExist, Candidate.MultipleObjectsReturned):
            return JsonResponse(
                {"message": "No such candidate for this voting"}, status=400
            )

    for i in range(0, empty_votes):
        candidate_objs.append(empty_candidate)

    try:
        mapping = TokenMapping.objects.get(token=token, voting=voting_obj)
    except TokenMapping.DoesNotExist:
        return JsonResponse({"message": "No uuid for token"}, status=403)

    # Double-check...
    cur_votes = Vote.objects.all().filter(uuid=mapping.uuid, voting=voting_obj)
    if len(cur_votes) != 0:
        return JsonResponse({"message": "Already voted in this voting!"}, status=403)

    for candidate_obj in candidate_objs:
        Vote(uuid=mapping.uuid, candidate=candidate_obj, voting=voting_obj).save()

    return votings_list(request)


@validate_token
@require_http_methods(["POST"])
def vote_transferable(request, token, voting_id):
    ## NEED TO CHECK THAT POSTS CORRECTLY
    if is_eligible_to_vote_transferable(request, voting_id) == False:
        return JsonResponse(
            {"message": "Not allowed to vote in this voting!"}, status=403
        )

    voting_obj = get_object_or_404(RankedChoiceVoting, pk=voting_id)
    token_obj = get_token_obj(request)

    candidates = []
    candidate_objs = {}
    vote_objs = []
    votes_noempty = []
    votes = []

    data = json.loads(request.body.decode("utf-8"))

    ### BEGIN VOTING PASSWORD CHECK ###
    voting_password_typed = data.get("voting_password")

    if not voting_password_typed:
        voting_password_typed = ""

    if is_valid_voting_password(voting_password_typed, voting_obj) == False:
        return JsonResponse({"message": "Wrong voting password!"}, status=403)
    ### END VOTING PASSWORD CHECK ###

    candidates = data.get("candidates")
    if not candidates:
        return JsonResponse({"message": "Candidates not provided"}, status=400)

    # Candi is pair of id:order
    for candidate in candidates:
        try:
            candidate_obj = RankedChoiceCandidate.objects.get(
                pk=candidate.split(":")[0], voting=voting_obj
            )
            if candidate.split(":")[1] != "-":
                candidate_objs[candidate.split(":")[1]] = candidate_obj
        except (
            RankedChoiceCandidate.DoesNotExist,
            RankedChoiceCandidate.MultipleObjectsReturned,
        ):
            return JsonResponse(
                {"message": "No such candidate for this voting"}, status=400
            )

    try:
        mapping = RankedChoiceTokenMapping.objects.get(
            token=token_obj, voting=voting_obj
        )
    except RankedChoiceTokenMapping.DoesNotExist:
        return JsonResponse({"message": "No uuid for token"}, status=403)

    # Double-check..

    # !!!!!!!!!! VERY IMPORTANT TODO!!!!!!!!

    cur_votes = RankedChoiceVoteGroup.objects.all().filter(
        uuid=mapping.uuid, voting=voting_obj
    )
    if len(cur_votes) != 0:
        return JsonResponse({"message": "Already voted in this voting!"}, status=403)

    # Create Vote group
    vote_group = RankedChoiceVoteGroup(
        uuid=mapping.uuid, voting=voting_obj, is_transferred=False
    )
    vote_group.save()

    for key in candidate_objs:
        RankedChoiceVote(
            uuid=mapping.uuid,
            candidate=candidate_objs[key],
            voting=voting_obj,
            preference=key,
            votegroup=vote_group,
        ).save()

    return votings_list(request)
