from .base import *

DEBUG = True

ALLOWED_HOSTS = ["localhost", "0.0.0.0"]

DATABASES = {
    "default": {
        "ENGINE": "django.db.backends.postgresql",
        "NAME": "vaalikoppi",
        "USER": "vaalikoppi",
        "PASSWORD": "secret",
        "HOST": "db",
        "PORT": "",
        "ATOMIC_REQUESTS": True,
    }
}

SHOW_DEBUG_TOOLBAR = True

if SHOW_DEBUG_TOOLBAR:
    # Show django debug toolbar always.
    # This is needed because the Docker internal IP is not static
    DEBUG_TOOLBAR_CONFIG = {
        "SHOW_TOOLBAR_CALLBACK": lambda request: True if DEBUG else False
    }
    INSTALLED_APPS += ("debug_toolbar",)
    MIDDLEWARE += ("debug_toolbar.middleware.DebugToolbarMiddleware",)
