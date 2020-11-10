import random

from django.shortcuts import render
from silk.profiling.profiler import silk_profile
from vaalikoppi.views.helpers import get_token_obj, is_valid_token
from vaalikoppi.views.votings import votings_list_data


@silk_profile(name='Index')
def index(request):
    data = {
        "is_valid_token": False,
        "user_alias": "",
    }

    cur_token_obj = get_token_obj(request)

    if is_valid_token(request, cur_token_obj):
        data["is_valid_token"] = True
        data["user_alias"] = cur_token_obj.alias

        voting_data = votings_list_data(request)
        data["is_admin"] = voting_data["is_admin"]
        data["closed_votings"] = voting_data["closed_votings"]
        data["open_votings"] = voting_data["open_votings"]
        data["ended_votings"] = voting_data["ended_votings"]

    return render(request, "index.html", data)
