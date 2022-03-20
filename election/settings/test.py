import nplusone

from .base import *

# Show warnings for possible N+1 queries
# nplusone.show_nplusones()

DEBUG = True

ALLOWED_HOSTS = ["localhost", "0.0.0.0", "kukka.digital"]

# Use different redis db for tests
CACHEOPS_REDIS = "redis://redis:6379/1"

# One hour caching by default
CACHEOPS_DEFAULTS = {"timeout": 60 * 60}
CACHEOPS = {
    "vaalikoppi.*": {"ops": "all", "cache_on_save": True},
}

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
