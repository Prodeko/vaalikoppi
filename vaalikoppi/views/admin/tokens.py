import json
import os.path
import random

from django.conf import settings
from django.contrib.auth.decorators import login_required
from django.http import JsonResponse
from django.shortcuts import get_object_or_404, render
from django.views.decorators.http import require_http_methods
from vaalikoppi.forms import *
from vaalikoppi.models import *


@login_required
def admin_tokens(request):
    all_tokens = list(Usertoken.objects.all())
    new_tokens = len([t for t in all_tokens if not t.activated and not t.invalidated])
    active_tokens = len([t for t in all_tokens if t.activated and not t.invalidated])
    invalid_tokens = len([t for t in all_tokens if t.invalidated])

    return render(
        request,
        "admin-tokens.html",
        {
            "tokens": all_tokens,
            "new_tokens": new_tokens,
            "active_tokens": active_tokens,
            "invalid_tokens": invalid_tokens,
        },
    )


@login_required
def print_tokens(request):
    tokens = Usertoken.objects.filter(activated=False, invalidated=False)
    return render(request, "tokens.html", {"tokens": tokens})


@login_required
@require_http_methods(["POST"])
def generate_tokens(request):
    data = json.loads(request.body.decode("utf-8"))
    count = data.get("count")
    if count:
        count = int(count)
        with open(os.path.join(settings.BASE_DIR, "wordlist.txt")) as f:
            words = [x.strip() for x in f]

        random_gen = random.SystemRandom()
        word_count = 4

        for _ in range(0, count):
            separator_int = random_gen.randint(0, 9)
            cur_token = str(separator_int).join(random_gen.sample(words, word_count))
            Usertoken(token=cur_token).save()

        return JsonResponse({"message": "success"}, status=200)
    return JsonResponse({"message": "Token count not provided"}, status=400)


@login_required
@require_http_methods(["POST"])
def activate_token(request):
    data = json.loads(request.body.decode("utf-8"))
    token = data.get("token")
    if token:
        token_obj = get_object_or_404(Usertoken, token=token)

        token_obj.activated = True
        token_obj.save()

        return JsonResponse({"message": "Success"}, status=200)
    return JsonResponse({"message": "Token not provided"}, status=400)


@login_required
@require_http_methods(["POST"])
def invalidate_token(request):
    data = json.loads(request.body.decode("utf-8"))
    token = data.get("token")
    if token:
        token_obj = get_object_or_404(Usertoken, token=token)

        token_obj.invalidated = True
        token_obj.save()

        return JsonResponse({"message": "Success"}, status=200)
    return JsonResponse({"message": "Token not provided"}, status=400)


@login_required
@require_http_methods(["POST"])
def invalidate_all_tokens(request):
    Usertoken.objects.all().filter(activated=True).update(invalidated=True)

    return JsonResponse({"message": "success"}, status=200)
