import json

from django.contrib.auth.decorators import login_required
from django.http import JsonResponse
from django.shortcuts import get_object_or_404, render
from django.views.decorators.http import require_http_methods
from py3votecore.stv import *
from vaalikoppi.forms import RankedChoiceVotingForm, VotingForm
from vaalikoppi.models import *
from vaalikoppi.views.helpers import votings_list_data


@login_required
def voting_results(request):
    votings = NormalVotingResult.objects.all()
    return render(request, "admin-voting-results.html", {"votings": votings})


@login_required
def admin_votings_list_data(request):
    data = votings_list_data(request, None, is_admin=True)
    active_tokens_count = Usertoken.objects.filter(
        activated=True, invalidated=False
    ).count()

    return {
        "is_admin": data["is_admin"],
        "closed_votings": data["closed_votings"],
        "open_votings": data["open_votings"],
        "ended_votings": data["ended_votings"],
        "active_tokens_count": active_tokens_count,
    }


@login_required
def admin_votings(request):
    return render(request, "admin-voting.html", admin_votings_list_data(request))


@login_required
def admin_voting_list(request):
    return render(request, "admin-voting-list.html", admin_votings_list_data(request))


@login_required
@require_http_methods(["POST"])
def create_voting(request):
    data = json.loads(request.body.decode("utf-8"))
    is_ranked_choice = data.get("is_ranked_choice")

    if is_ranked_choice:
        RankedChoiceVotingForm(data).save()
    else:
        VotingForm(data).save()

    return JsonResponse({"is_ranked_choice": is_ranked_choice}, status=200)


@login_required
@require_http_methods(["POST"])
def add_candidate(request, voting_id):
    data = json.loads(request.body.decode("utf-8"))
    is_ranked_choice = data.get("is_ranked_choice")
    candidate_name = data.get("candidate_name")

    if is_ranked_choice:
        voting = get_object_or_404(RankedChoiceVoting, pk=voting_id)
        candidate = RankedChoiceCandidate(voting=voting, candidate_name=candidate_name)
        candidate.save()
    else:
        voting = get_object_or_404(NormalVoting, pk=voting_id)
        candidate = NormalCandidate(voting=voting, candidate_name=candidate_name)
        candidate.save()

    return JsonResponse({"message": "success"}, status=200)


@login_required
@require_http_methods(["POST"])
def remove_candidate(request, candidate_id):
    data = json.loads(request.body.decode("utf-8"))
    is_ranked_choice = data.get("is_ranked_choice")

    if is_ranked_choice:
        get_object_or_404(RankedChoiceCandidate, pk=candidate_id).delete()
    else:
        get_object_or_404(NormalCandidate, pk=candidate_id).delete()

    return JsonResponse({"message": "success"}, status=200)


@login_required
@require_http_methods(["POST"])
def open_voting(request, voting_id):
    data = json.loads(request.body.decode("utf-8"))
    is_ranked_choice = data.get("is_ranked_choice")

    if is_ranked_choice:
        voting_obj = get_object_or_404(RankedChoiceVoting, pk=voting_id)
    else:
        voting_obj = get_object_or_404(NormalVoting, pk=voting_id)
        NormalCandidate(
            candidate_name="Tyhjä", empty_candidate=True, voting=voting_obj
        ).save()

    if voting_obj.is_open or voting_obj.is_ended:
        return JsonResponse(
            {"message": "Voting is already open or has ended"}, status=403
        )

    active_tokens = Usertoken.objects.filter(activated=True, invalidated=False)
    if is_ranked_choice:
        for cur_token in active_tokens:
            RankedChoiceTokenMapping(token=cur_token, voting=voting_obj).save()
    else:
        for cur_token in active_tokens:
            NormalTokenMapping(token=cur_token, voting=voting_obj).save()

    voting_obj.open_voting()
    return JsonResponse({"message": "Voting opened"}, status=200)


def transfer_election_has_result(request, voting_obj):
    candidates = RankedChoiceCandidate.objects.all().filter(voting=voting_obj)
    for candidate in candidates:
        if not candidate.has_dropped and not candidate.chosen:
            return False
    return True


@login_required
@require_http_methods(["POST"])
def close_voting(request, voting_id):
    data = json.loads(request.body.decode("utf-8"))
    is_ranked_choice = data.get("is_ranked_choice")

    def calc_vote_share(vote_count, tot_votes_abs):
        if tot_votes_abs > 0:
            percentage_of_votes = round(100 * vote_count / tot_votes_abs, 1)
            return f"{percentage_of_votes}"
        return "0.0"

    if is_ranked_choice:
        voting_obj = get_object_or_404(RankedChoiceVoting, pk=voting_id)
    else:
        voting_obj = get_object_or_404(NormalVoting, pk=voting_id)

    if not voting_obj.is_open or voting_obj.is_ended:
        return JsonResponse({"message": "Voting is not open or has ended"}, status=403)

    if is_ranked_choice:
        mappings = RankedChoiceTokenMapping.objects.all().filter(voting=voting_obj)
    else:
        mappings = NormalTokenMapping.objects.all().filter(voting=voting_obj)

    for mapping in mappings:
        if is_ranked_choice:
            cur_votes = RankedChoiceVote.objects.all().filter(
                uuid=mapping.uuid, voting=voting_obj
            )
            cur_votegroups = RankedChoiceVoteGroup.objects.all().filter(
                uuid=mapping.uuid, voting=voting_obj
            )
            candidates = RankedChoiceCandidate.objects.all().filter(voting=voting_obj)
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

            RankedChoiceVotingVoterStatus(
                voting=voting_obj,
                usertoken_token=mapping.token.token,
                usertoken_alias=mapping.token.alias,
                has_voted=has_voted,
            ).save()
        else:
            cur_votes = NormalVote.objects.all().filter(
                uuid=mapping.uuid, voting=voting_obj
            )
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

            NormalVotingVoterStatus(
                voting=voting_obj,
                usertoken_token=mapping.token.token,
                usertoken_alias=mapping.token.alias,
                has_voted=has_voted,
            ).save()

        mapping.delete()

    if is_ranked_choice:
        results = calculate_results_stv(request, voting_obj)
        for voting_round in results["rounds"]:
            for candidate in voting_round["candidates"]:
                RankedChoiceVotingResult(
                    voting=voting_obj,
                    candidate_name=candidate["name"],
                    vote_count=candidate["vote_count"],
                    elected=candidate["elected"],
                    dropped=candidate["dropped"],
                    vote_rounds=voting_round["round"],
                ).save()
        voting_obj.voting_round = len(results["rounds"])
        voting_obj.save()
    else:
        for cur_candidate in voting_obj.candidates.all():
            cur_vote_count = len(
                NormalVote.objects.all().filter(
                    voting=voting_obj, candidate=cur_candidate
                )
            )
            cur_vote_share = calc_vote_share(
                cur_vote_count, voting_obj.total_votes_abs()
            )
            NormalVotingResult(
                voting=voting_obj,
                candidate_name=cur_candidate.candidate_name,
                vote_count=cur_vote_count,
                vote_share=cur_vote_share,
            ).save()

    voting_obj.close_voting()

    return JsonResponse({"message": "Voting closed"}, status=200)


def calculate_results_stv(request, voting_obj):
    inputs = calculate_stv(request, voting_obj.id)

    stv_success = False
    cur_max_votes = voting_obj.max_votes
    results = {}

    # For cases with less voted candidates than candidates intended to be selected:
    # Iteratively lower max_votes to reach a number that actually can be selected
    # with the votes given.
    # Eventually, when max_votes=0 (no votes given at all), no one gets selected and this allows  
    # to close an empty voting (really, I promise..).
    while(not(stv_success) and cur_max_votes >= 0):
        try:
            results = STV(inputs, required_winners=cur_max_votes).as_dict()
            stv_success = True
        except:
            cur_max_votes -= 1
    
    counter = 1
    for voting_round in results["rounds"]:
        voting_round["round"] = counter
        voting_round["candidates"] = []

        for person in voting_round["tallies"]:
            obj = {}
            obj["id"] = person
            obj["name"] = RankedChoiceCandidate.objects.get(id=person).candidate_name
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
    countdict = {}
    keysdict = {}

    voting_obj = get_object_or_404(RankedChoiceVoting, pk=voting_id)
    votegroups = RankedChoiceVoteGroup.objects.all().filter(voting=voting_obj)
    for vote_group in votegroups:
        votes = (
            RankedChoiceVote.objects.all()
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
