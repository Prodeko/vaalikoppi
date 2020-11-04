import json

from django.conf import settings
from django.http import JsonResponse
from django.shortcuts import get_object_or_404
from django.views.decorators.http import require_http_methods
from py3votecore.stv import *
from vaalikoppi.forms import *
from vaalikoppi.models import *
from vaalikoppi.views.helpers import is_valid_token, validate_register_alias


@require_http_methods(["POST"])
def user_login(request):
    data = json.loads(request.body.decode("utf-8"))
    token = data.get("token")
    alias = data.get("alias")
    
    if token:
        try:
            token_obj = Usertoken.objects.get(token=token)
        except:
            return JsonResponse({"message": "Invalid token"}, status=401)
        
        if token_obj.activated and not token_obj.invalidated:
            try:
                validate_register_alias(request, token_obj, alias)
            except:
                return JsonResponse({"message": "Alias not available"}, status=403)
            
            request.session[settings.USER_TOKEN_VAR] = token_obj.token
            return JsonResponse({"message": "Login success", "token": token_obj.token, "alias": token_obj.alias}, status=200)
            
        else:
            # Do not tell the user why code validation fails
            return JsonResponse({"message": "Invalid token"}, status=401)
            
    return JsonResponse({"message": "Token not provided"}, status=400)


def user_logout(request):
    request.session[settings.USER_TOKEN_VAR] = ""
    request.session.flush()

    if is_valid_token(request) == False:
        return JsonResponse({"message": "Logged out", "status": 0}, status=200)

    return JsonResponse({"message": "Could not log out"}, status=500)
