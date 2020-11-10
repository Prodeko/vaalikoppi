import json

from django.contrib.auth.decorators import login_required
from django.http import JsonResponse
from django.shortcuts import get_object_or_404, render
from django.views.decorators.http import require_http_methods
from py3votecore.stv import *
from vaalikoppi.forms import *
from vaalikoppi.models import *
from vaalikoppi.views.helpers import get_active_tokens


@login_required
def voting_results(request):
    votings = VotingResult.objects.all()
    return render(request, "admin-voting-results.html", {"votings": votings})


@login_required
def admin_votings(request):
    return render(request, "admin-voting.html")


@login_required
@require_http_methods(["POST"])
def create_voting(request):
    data = json.loads(request.body.decode("utf-8"))
    is_transferable = data.get("is_transferable")

    if is_transferable:
        VotingTransferableForm(data).save()
    else:
        VotingForm(data).save()

    return JsonResponse({"is_transferable": is_transferable}, status=200)


@login_required
@require_http_methods(["POST"])
def add_candidate(request, voting_id):
    data = json.loads(request.body.decode("utf-8"))
    is_transferable = data.get("is_transferable")
    candidate_name = data.get("candidate_name")

    if is_transferable:
        voting = get_object_or_404(VotingTransferable, pk=voting_id)
        candidate = CandidateTransferable(voting=voting, candidate_name=candidate_name)
        candidate.save()
    else:
        voting = get_object_or_404(Voting, pk=voting_id)
        candidate = Candidate(voting=voting, candidate_name=candidate_name)
        candidate.save()

    return JsonResponse({"message": "success"}, status=200)


@login_required
@require_http_methods(["POST"])
def remove_candidate(request, candidate_id):
    data = json.loads(request.body.decode("utf-8"))
    is_transferable = data.get("is_transferable")

    if is_transferable:
        get_object_or_404(CandidateTransferable, pk=candidate_id).delete()
    else:
        get_object_or_404(Candidate, pk=candidate_id).delete()

    return JsonResponse({"message": "success"}, status=200)


@login_required
@require_http_methods(["POST"])
def open_voting(request, voting_id):
    data = json.loads(request.body.decode("utf-8"))
    is_transferable = data.get("is_transferable")

    if is_transferable:
        voting_obj = get_object_or_404(VotingTransferable, pk=voting_id)
    else:
        voting_obj = get_object_or_404(Voting, pk=voting_id)
        Candidate(
            candidate_name="Tyhjä", empty_candidate=True, voting=voting_obj
        ).save()

    if voting_obj.is_open or voting_obj.is_ended:
        return JsonResponse(
            {"message": "Voting is already open or has ended"}, status=403
        )

    active_tokens = get_active_tokens(request)
    if is_transferable:
        for cur_token in active_tokens:
            TokenMappingTransferable(token=cur_token, voting=voting_obj).save()
    else:
        for cur_token in active_tokens:
            TokenMapping(token=cur_token, voting=voting_obj).save()

    voting_obj.open_voting()
    return JsonResponse({"message": "Voting opened"}, status=200)


def transfer_election_has_result(request, voting_obj):
    candidates = CandidateTransferable.objects.all().filter(voting=voting_obj)
    for candidate in candidates:
        if candidate.has_dropped == False and candidate.chosen == False:
            return False
    return True


@login_required
@require_http_methods(["POST"])
def close_voting_transferable(request, voting_id):
    voting_obj = get_object_or_404(Voting, pk=voting_id)
    not_voted_tokens = []

    if not voting_obj.is_open or voting_obj.is_ended:
        return JsonResponse({"message": "Voting is not open or has ended"}, status=403)

    for mapping in TokenMapping.objects.all().filter(voting=voting_obj):
        cur_votes = Vote.objects.all().filter(uuid=mapping.uuid, voting=voting_obj)
        if len(cur_votes) > voting_obj.max_votes:
            return JsonResponse(
                {
                    "message": "Security compromised - too many votes from a single voter"
                },
                status=500,
            )
        if len(cur_votes) == 0:
            not_voted_tokens.append(mapping.get_token().token)

    voting_obj.close_voting()

    quota = (
        len(TokenMapping.objects.all().filter(voting=voting_obj))
        / (voting_obj.max_votes + 1)
        + 1
    )

    TokenMapping.objects.all().filter(voting=voting_obj).delete()

    while not transfer_election_has_result(request, voting_obj):
        # to do implement iterative LookupError
        continue


@login_required
@require_http_methods(["POST"])
def close_voting(request, voting_id):
    data = json.loads(request.body.decode("utf-8"))
    is_transferable = data.get("is_transferable")

    if is_transferable:
        voting_obj = get_object_or_404(VotingTransferable, pk=voting_id)
    else:
        voting_obj = get_object_or_404(Voting, pk=voting_id)

    if not voting_obj.is_open or voting_obj.is_ended:
        return JsonResponse({"message": "Voting is not open or has ended"}, status=403)

    if is_transferable:
        mappings = TokenMappingTransferable.objects.all().filter(voting=voting_obj)
    else:
        mappings = TokenMapping.objects.all().filter(voting=voting_obj)

    for mapping in mappings:
        if is_transferable:
            cur_votes = VoteTransferable.objects.all().filter(
                uuid=mapping.uuid, voting=voting_obj
            )
            cur_votegroups = VoteGroupTransferable.objects.all().filter(
                uuid=mapping.uuid, voting=voting_obj
            )
            candidates = CandidateTransferable.objects.all().filter(voting=voting_obj)
            has_voted = True

            if len(cur_votes) > len(candidates):
                return JsonResponse(
                    {
                        "message": "Security compromised - too many votes from a single voter"
                    },
                    status=500,
                )
            if len(cur_votegroups) > 1:
                return JsonResponse(
                    {
                        "message": "Security compromised - too many votes from a single voter"
                    },
                    status=500,
                )
            if len(cur_votegroups) == 0:
                has_voted = False

            status = VoterStatusTransferable(
                voting=voting_obj,
                usertoken_token=mapping.token.token,
                usertoken_alias=mapping.token.alias,
                has_voted=has_voted,
            ).save()
        else:
            cur_votes = Vote.objects.all().filter(uuid=mapping.uuid, voting=voting_obj)
            has_voted = True

            if len(cur_votes) > voting_obj.max_votes:
                return JsonResponse(
                    {
                        "message": "Security compromised - too many votes from a single voter"
                    },
                    status=500,
                )
            if len(cur_votes) == 0:
                has_voted = False

            status = VoterStatusRegular(
                voting=voting_obj,
                usertoken_token=mapping.token.token,
                usertoken_alias=mapping.token.alias,
                has_voted=has_voted,
            ).save()

        mapping.delete()

    if is_transferable:
        results = calculate_results_stv(request, voting_obj)
        for voting_round in results["rounds"]:
            for candidate in voting_round["candidates"]:
                VotingResultTransferable(
                    voting=voting_obj,
                    candidate_name=candidate["name"],
                    vote_count=candidate["vote_count"],
                    elected=candidate["elected"],
                    dropped=candidate["dropped"],
                    vote_rounds=voting_round["round"],
                ).save()
        voting_obj.round = len(results["rounds"])
        voting_obj.save()
    else:
        for cur_candidate in voting_obj.candidates.all():
            cur_vote_count = len(
                Vote.objects.all().filter(voting=voting_obj, candidate=cur_candidate)
            )
            VotingResult(
                voting=voting_obj,
                candidate_name=cur_candidate.candidate_name,
                vote_count=cur_vote_count,
            ).save()

    voting_obj.close_voting()

    return JsonResponse({"message": "Voting closed"}, status=200)


def calculate_results_stv(request, voting_obj):
    inputs = calculate_stv(request, voting_obj.id)
    results = STV(inputs, required_winners=voting_obj.max_votes).as_dict()

    counter = 1
    for voting_round in results["rounds"]:
        voting_round["round"] = counter
        voting_round["candidates"] = []

        for person in voting_round["tallies"]:
            obj = {}
            obj["id"] = person
            obj["name"] = CandidateTransferable.objects.get(id=person).candidate_name
            obj["vote_count"] = voting_round["tallies"][person]
            voting_round["candidates"].append(obj)
            obj["elected"] = False
            obj["dropped"] = False

            if "winners" in voting_round and person in voting_round["winners"]:
                obj["elected"] = True
            if "loser" in voting_round:
                if voting_round["loser"] == person:
                    obj["dropped"] = True
                elif len(voting_round["tallies"]) == 2:
                    obj["elected"] = True

        voting_round["candidates"] = sorted(
            voting_round["candidates"], key=lambda k: k["vote_count"], reverse=True
        )

        del voting_round["tallies"]
        if "loser" in voting_round:
            del voting_round["loser"]
        if "winners" in voting_round:
            del voting_round["winners"]
        counter += 1

    return results


def calculate_stv(request, voting_id):
    ballots = []
    ballots2 = []
    countdict = {}
    keysdict = {}

    voting_obj = get_object_or_404(VotingTransferable, pk=voting_id)
    votegroups = VoteGroupTransferable.objects.all().filter(voting=voting_obj)
    for vote_group in votegroups:
        votes = (
            VoteTransferable.objects.all()
            .filter(votegroup=vote_group)
            .order_by("preference")
        )
        vote_array = []
        keystring = ""
        for vote in votes:
            keystring += str(vote.candidate) + "-" + str(vote.preference)
            vote_array.append(str(vote.candidate.id))
        if keystring in countdict:
            countdict[keystring] = countdict[keystring] + 1
        else:
            keysdict[keystring] = vote_array
            countdict[keystring] = 1

    for key, value in keysdict.items():
        ballots.append({"count": countdict[key], "ballot": value})

    return ballots


@login_required
def admin_voting_list(request):
    closed_regular_votings = list(Voting.objects.filter(is_open=False, is_ended=False))
    closed_transferable_votings = list(
        VotingTransferable.objects.filter(is_open=False, is_ended=False)
    )
    closed_votings = sorted(
        (closed_regular_votings + closed_transferable_votings),
        key=lambda v: v.pseudo_unique_id(),
        reverse=True,
    )

    open_regular_votings = list(Voting.objects.filter(is_open=True, is_ended=False))

    # Attach voted and not voted tokens to the respective votings
    # Regular votings
    for open_regular_voting in open_regular_votings:
        cur_mappings = TokenMapping.objects.all().filter(voting=open_regular_voting)

        open_regular_voting.tokens_voted = []
        open_regular_voting.tokens_not_voted = []
        for cur_mapping in cur_mappings:
            cur_votes_count = (
                Vote.objects.all()
                .filter(uuid=cur_mapping.uuid, voting=open_regular_voting)
                .count()
            )

            if cur_votes_count > 0:
                open_regular_voting.tokens_voted.append(cur_mapping.token)
            else:
                open_regular_voting.tokens_not_voted.append(cur_mapping.token)

    open_transferable_votings = list(
        VotingTransferable.objects.filter(is_open=True, is_ended=False)
    )

    # Attach voted and not voted tokens to the respective votings
    # Transferable votings
    for open_transferable_voting in open_transferable_votings:
        cur_mappings = TokenMappingTransferable.objects.all().filter(
            voting=open_transferable_voting
        )

        open_transferable_voting.tokens_voted = []
        open_transferable_voting.tokens_not_voted = []
        for cur_mapping in cur_mappings:
            cur_votes_count = (
                VoteTransferable.objects.all()
                .filter(uuid=cur_mapping.uuid, voting=open_transferable_voting)
                .count()
            )

            if cur_votes_count > 0:
                open_transferable_voting.tokens_voted.append(cur_mapping.token)
            else:
                open_transferable_voting.tokens_not_voted.append(cur_mapping.token)

    # Combine regular and transferable votings into one list
    open_votings = sorted(
        (open_regular_votings + open_transferable_votings),
        key=lambda v: v.pseudo_unique_id(),
        reverse=True,
    )

    ended_regular_votings = list(Voting.objects.filter(is_open=False, is_ended=True))
    ended_transferable_votings = list(
        VotingTransferable.objects.filter(is_open=False, is_ended=True)
    )
    ended_votings = sorted(
        (ended_regular_votings + ended_transferable_votings),
        key=lambda v: v.pseudo_unique_id(),
        reverse=True,
    )

    active_tokens_count = len(get_active_tokens(request))

    return render(
        request,
        "admin-voting-list.html",
        {
            "is_admin": True,
            "closed_votings": closed_votings,
            "open_votings": open_votings,
            "ended_votings": ended_votings,
            "active_tokens_count": active_tokens_count,
        },
    )
