import json

from django.conf import settings
from django.http import JsonResponse
from django.shortcuts import get_object_or_404
from django.views.decorators.http import require_http_methods
from py3votecore.stv import *
from vaalikoppi.forms import *
from vaalikoppi.models import *
from vaalikoppi.views.helpers import is_valid_token


@require_http_methods(["POST"])
def user_login(request):
    data = json.loads(request.body.decode("utf-8"))
    token = data.get("token")
    if token:
        token_obj = get_object_or_404(Usertoken, token=token)

        if token_obj.activated and not token_obj.invalidated:
            request.session[settings.USER_TOKEN_VAR] = token_obj.token
            return JsonResponse(
                {"message": "Login success", "token": token_obj.token}, status=200
            )
        else:
            return JsonResponse({"message": "Invalid token"}, status=403)

        return JsonResponse({"message": "Success"}, status=200)
    return JsonResponse({"message": "Token not provided"}, status=400)


def user_logout(request):
    request.session[settings.USER_TOKEN_VAR] = ""
    request.session.flush()

    if is_valid_token(request) == False:
        return JsonResponse({"message": "Logged out", "status": 0}, status=200)

    return JsonResponse({"message": "Could not log out"}, status=500)
