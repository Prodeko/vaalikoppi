import random

from django.shortcuts import render
from vaalikoppi.views.helpers import is_valid_token


def index(request):
    data = {
        "is_valid_token": False,
        "nocache_rand": random.randint(10000, 99999),
    }

    if is_valid_token(request):
        data["is_valid_token"] = True

    return render(request, "index.html", data)
