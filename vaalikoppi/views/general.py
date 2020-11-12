from django.shortcuts import render

# Set SHOW_DJANGO_SILK = True in election.settings.dev and
# uncomment silk import and @silk_profile to enable django-silk
# from silk.profiling.profiler import silk_profile
from vaalikoppi.views.helpers import get_token_from_session, is_valid_token
from vaalikoppi.views.votings import votings_list_data


# @silk_profile(name="Index")
def index(request):
    data = {
        "is_valid_token": False,
        "user_alias": "",
    }

    token = get_token_from_session(request)
    data = votings_list_data(request, token)

    if token:
        data["is_valid_token"] = True
        data["user_alias"] = token.alias

    return render(request, "index.html", data)
