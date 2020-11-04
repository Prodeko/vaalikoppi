from django.conf import settings
from vaalikoppi.models import Usertoken


def is_valid_token(request, token_obj=None):
    if token_obj is None:
        token_obj = get_token_obj(request)

    if token_obj is not None and token_obj.activated and not token_obj.invalidated:
        return True

    return False


def get_token_obj(request):
    session_var_name = settings.USER_TOKEN_VAR

    if session_var_name in request.session:
        cur_token = request.session[session_var_name]

        try:
            token_obj = Usertoken.objects.get(token=cur_token)
            return token_obj
        except Usertoken.DoesNotExist:
            return None

    return None


def get_active_tokens(request):
    return Usertoken.objects.filter(activated=True, invalidated=False)


# Raises exception
def validate_register_alias(request, token_obj, alias):
    alias_len = len(alias)
     
    if alias_len >=3 and alias_len <=20 and 0 == get_active_tokens(request).filter(alias=alias).count():
        Usertoken.objects.filter(token=token_obj.token).update(alias=alias)
    else:
        raise Exception("Invalid alias provided")