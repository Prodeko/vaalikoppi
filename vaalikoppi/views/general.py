from django.shortcuts import render
# Set SHOW_DJANGO_SILK = True in election.settings.dev and
# uncomment silk import and @silk_profile to enable django-silk
# from silk.profiling.profiler import silk_profile
from vaalikoppi.views.helpers import get_token_from_session
from vaalikoppi.views.votings import votings_list_data


# @silk_profile(name="Index")
def index(request):
    data = {
        "is_valid_token": False,
        "user_alias": "",
    }

    token = get_token_from_session(request)
    token_is_valid = token is not None and token.activated and not token.invalidated

    # Do not even attempt to fetch votings in the case of an invalid token
    # in order to save valuable database queries
    if token_is_valid:
        data = votings_list_data(request, token)
        data["is_valid_token"] = token_is_valid
        data["user_alias"] = token.alias

    return render(request, "index.html", data)
