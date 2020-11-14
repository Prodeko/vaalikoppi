import json

from django.http import JsonResponse
from django.shortcuts import get_object_or_404, render
from django.views.decorators.http import require_http_methods
from py3votecore.stv import *
from vaalikoppi.models import *
from vaalikoppi.views.helpers import (
    is_eligible_to_vote_normal,
    is_eligible_to_vote_ranked_choice,
    validate_token,
    votings_list_data,
)


def is_valid_voting_password(voting_password_typed, voting_obj):
    voting_password_real = voting_obj.voting_password
    voting_requires_password = voting_obj.is_password_protected

    if not voting_requires_password:
        return True

    if voting_password_typed == voting_password_real:
        return True
    return False


# Get votings list data directly


# Get votings list rendered as html
@validate_token
def votings_list(request, token):
    return render(request, "voting-list.html", votings_list_data(request, token))


@validate_token
@require_http_methods(["POST"])
def vote_normal(request, token, voting_id):
    voting_obj = get_object_or_404(NormalVoting, pk=voting_id)

    is_eligible = is_eligible_to_vote_normal(token, voting_obj)

    if not is_eligible:
        return JsonResponse(
            {"message": "Not allowed to vote in this voting!"}, status=403
        )

    candidates = []
    candidates_noempty = []
    candidate_objs = []
    empty_candidate = NormalCandidate.objects.get(
        voting=voting_obj, empty_candidate=True
    )

    data = json.loads(request.body.decode("utf-8"))

    ### BEGIN VOTING PASSWORD CHECK ###
    voting_password_typed = data.get("voting_password")

    if not voting_password_typed:
        voting_password_typed = ""

    if not is_valid_voting_password(voting_password_typed, voting_obj):
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
            candidate_obj = NormalCandidate.objects.get(
                pk=candidate_id, voting=voting_obj
            )
            candidate_objs.append(candidate_obj)
        except (NormalCandidate.DoesNotExist, NormalCandidate.MultipleObjectsReturned):
            return JsonResponse(
                {"message": "No such candidate for this voting"}, status=400
            )

    for i in range(0, empty_votes):
        candidate_objs.append(empty_candidate)

    try:
        mapping = NormalTokenMapping.objects.get(token=token, voting=voting_obj)
    except NormalTokenMapping.DoesNotExist:
        return JsonResponse({"message": "No uuid for token"}, status=403)

    # Double-check...
    cur_votes = NormalVote.objects.all().filter(uuid=mapping.uuid, voting=voting_obj)
    if len(cur_votes) != 0:
        return JsonResponse({"message": "Already voted in this voting!"}, status=403)

    for candidate_obj in candidate_objs:
        NormalVote(uuid=mapping.uuid, candidate=candidate_obj, voting=voting_obj).save()

    return votings_list(request)


@validate_token
@require_http_methods(["POST"])
def vote_ranked_choice(request, token, voting_id):
    # NEED TO CHECK THAT POSTS CORRECTLY
    voting_obj = get_object_or_404(RankedChoiceVoting, pk=voting_id)
    is_eligible = is_eligible_to_vote_ranked_choice(token, voting_obj)
    if not is_eligible:
        return JsonResponse(
            {"message": "Not allowed to vote in this voting!"}, status=403
        )

    candidates = []
    candidate_objs = {}

    data = json.loads(request.body.decode("utf-8"))

    ### BEGIN VOTING PASSWORD CHECK ###
    voting_password_typed = data.get("voting_password")

    if not voting_password_typed:
        voting_password_typed = ""

    if not is_valid_voting_password(voting_password_typed, voting_obj):
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
        mapping = RankedChoiceTokenMapping.objects.get(token=token, voting=voting_obj)
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
