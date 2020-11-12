import json

from django.conf import settings
from django.http import JsonResponse
from django.views.decorators.http import require_http_methods
from vaalikoppi.models import Usertoken
from vaalikoppi.views.helpers import AliasException, validate_register_alias


@require_http_methods(["POST"])
def user_login(request):
    data = json.loads(request.body.decode("utf-8"))
    token = data.get("token")
    alias = data.get("alias")

    # POST data has to include token and alias
    if token and alias:
        # Try to find matching Usertoken object
        try:
            token_obj = Usertoken.objects.get(token=token)
        except Usertoken.DoesNotExist:
            # Token not found
            return JsonResponse({"message": "Invalid token"}, status=401)

        # Token has to be activate and not invalidated
        if token_obj.activated and not token_obj.invalidated:
            try:
                validate_register_alias(request, token_obj, alias)
            except AliasException:
                # Alias already in use
                return JsonResponse({"message": "Alias not available"}, status=403)

            # Store token in session
            request.session[settings.USER_TOKEN_VAR] = token_obj.token

            # Login success
            return JsonResponse(
                {
                    "message": "Login success",
                    "token": token_obj.token,
                    "alias": token_obj.alias,
                },
                status=200,
            )
        return JsonResponse({"message": "Invalid token"}, status=401)
    return JsonResponse({"message": "Token or alias not provided"}, status=400)


def user_logout(request):
    request.session[settings.USER_TOKEN_VAR] = ""
    request.session.flush()

    return JsonResponse({"message": "Logged out", "status": 0}, status=200)
