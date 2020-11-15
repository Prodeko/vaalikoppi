import json

from django.http import JsonResponse
from django.shortcuts import get_object_or_404, render
from django.views.decorators.http import require_http_methods
from vaalikoppi.exceptions import JsonResponseException
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


@validate_token
def votings_list(request, token):
    return render(request, "voting-list.html", votings_list_data(request, token))


def voting_common_checks(token, voting, submitted_password, candidates):
    is_eligible = is_eligible_to_vote_normal(token, voting)

    # Check eligibility to vote
    if not is_eligible:
        raise JsonResponseException("Not allowed to vote in this voting!", 403)

    # Voting password check
    if not is_valid_voting_password(submitted_password, voting):
        raise JsonResponseException("Wrong voting password!", 403)

    # Candidates data must be provided
    if not candidates:
        raise JsonResponseException("Candidates not provided.", 400)


@validate_token
@require_http_methods(["POST"])
def vote_normal(request, token, voting_id):
    """ Cast a vote in a normal voting."""
    try:
        # Load post data
        data = json.loads(request.body.decode("utf-8"))
        data_submitted_password = data.get("voting_password", "")
        data_candidates = data.get("candidates")
        voting = get_object_or_404(NormalVoting, pk=voting_id)

        # Run common validity checks
        voting_common_checks(token, voting, data_submitted_password, data_candidates)

        # Construct helper data
        empty_candidate = NormalCandidate.objects.get(
            voting=voting, empty_candidate=True
        )
        candidates_noempty = [
            x for x in data_candidates if x != str(empty_candidate.id)
        ]
        empty_votes = voting.max_votes - len(candidates_noempty)

        # Can't vote for the same candidate multiple times
        if len(candidates_noempty) != len(set(candidates_noempty)):
            raise JsonResponseException("Multiple votes for same candidate.", 403)

        # Check if provided candidate ids are valid for this election
        candidates = list(
            NormalCandidate.objects.filter(pk__in=data_candidates, voting=voting)
        )
        candidate_ids = [str(c.id) for c in candidates]

        # If data_candidates contains any ids not found in database return error
        if set(candidate_ids) != set(data_candidates):
            raise JsonResponseException("No such candidate found.", 403)

        # Append empty votes to candidates list
        for _ in range(1, empty_votes):
            candidates.append(empty_candidate)

        # Try to get token mapping connecting a token to this voting
        try:
            mapping = NormalTokenMapping.objects.get(token=token, voting=voting)
        except NormalTokenMapping.DoesNotExist:
            raise JsonResponseException("No uuid for token.", 403)

        # Double-check that a NormalVote object does not already exist for this token
        token_votes = NormalVote.objects.all().filter(uuid=mapping.uuid, voting=voting)
        if len(token_votes) != 0:
            raise JsonResponseException("Already voted in this voting!", 403)

        for candidate in candidates:
            NormalVote(uuid=mapping.uuid, candidate=candidate, voting=voting).save()

        # Render the voting list
        return votings_list(request)

    # Capture raised JsonResponseExceptions and return JsonResponse
    except JsonResponseException as e:
        return JsonResponse({"message": e.message}, status=e.status)


@validate_token
@require_http_methods(["POST"])
def vote_ranked_choice(request, token, voting_id):
    """ Cast a vote in a ranked choice voting."""
    try:
        # Load post data
        data = json.loads(request.body.decode("utf-8"))
        data_submitted_password = data.get("voting_password", "")
        data_candidates = data.get("candidates")
        voting = get_object_or_404(RankedChoiceVoting, pk=voting_id)
        candidates_and_preferences = {
            int(c.split(":")[0]): {"preference": c.split(":")[1]}
            for c in data_candidates if "-" not in c
        }
        data_candidate_ids = candidates_and_preferences.keys()

        # Run common validity checks
        voting_common_checks(token, voting, data_submitted_password, data_candidate_ids)

        # Check if provided candidate ids are valid for this election
        candidate_objects = list(
            RankedChoiceCandidate.objects.filter(
                pk__in=data_candidate_ids, voting=voting
            )
        )
        candidate_ids = []

        for c in candidate_objects:
            candidate_ids.append(c.id)
            candidates_and_preferences[c.id]["model"] = c

        # If data_candidates contains any ids not found in database return error
        if set(candidate_ids) != set(data_candidate_ids):
            raise JsonResponseException("No such candidate found.", 403)

        # Try to get token mapping connecting a token to this voting
        try:
            mapping = RankedChoiceTokenMapping.objects.get(token=token, voting=voting)
        except RankedChoiceTokenMapping.DoesNotExist:
            raise JsonResponseException("No uuid for token.", 403)

        # Double-check that a NormalVote object does not already exist for this token
        cur_votes = RankedChoiceVoteGroup.objects.all().filter(
            uuid=mapping.uuid, voting=voting
        )
        if len(cur_votes) != 0:
            raise JsonResponseException("Already voted in this voting!", 403)

        # Create Vote group
        vote_group = RankedChoiceVoteGroup(
            uuid=mapping.uuid, voting=voting, is_transferred=False
        )
        vote_group.save()

        for x in candidates_and_preferences.values():
            RankedChoiceVote(
                uuid=mapping.uuid,
                candidate=x["model"],
                voting=voting,
                preference=x["preference"],
                votegroup=vote_group,
            ).save()

        # Render the voting list
        return votings_list(request)

    # Capture raised JsonResponseExceptions and return JsonResponse
    except JsonResponseException as e:
        return JsonResponse({"message": e.message}, status=e.status)
