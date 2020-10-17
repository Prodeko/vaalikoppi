from django.conf import settings
from django.contrib.auth import SESSION_KEY as USER_ID_SESSION_KEY
from django.core.exceptions import MiddlewareNotUsed
from django.utils.deprecation import MiddlewareMixin


class SessionLockMiddleware(MiddlewareMixin):
    """
    Prevents users from making simultaneous requests.
    """

    def __init__(self, get_response):
        if not getattr(settings, "SESSION_LOCK", True):
            raise MiddlewareNotUsed

        super().__init__(get_response)

        from django.contrib.sessions.middleware import SessionMiddleware
        from django.db import DEFAULT_DB_ALIAS, connections

        self.connections = connections
        self.db_alias = getattr(settings, "SESSION_LOCK_DB", DEFAULT_DB_ALIAS)

    def process_request(self, request):
        # Generate a lock id.
        user_token = request.session.get(settings.USER_TOKEN_VAR)
        if user_token is not None:
            request.session_lock_id = ("user_lock_%s" % user_token).__hash__()
        else:
            # If user is anonymous then use meta info for identification.
            request.session_lock_id = (
                request.META.get("HTTP_HOST", "")
                + ":"
                + request.META.get("HTTP_USER_AGENT", "")
            ).__hash__()

        # Acquire the lock.
        cursor = self.connections[self.db_alias].cursor()
        cursor.execute("SELECT pg_advisory_lock(%d)" % request.session_lock_id)

    def process_response(self, request, response):
        self._release_lock(request)
        return response

    def process_exception(self, request, exception):
        self._release_lock(request)

    def _release_lock(self, request):
        if hasattr(request, "session_lock_id"):
            cursor = self.connections[self.db_alias].cursor()
            cursor.execute("SELECT pg_advisory_unlock(%d)" % request.session_lock_id)
