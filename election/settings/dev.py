from .base import *

DEBUG = True

ALLOWED_HOSTS = []

DATABASES = {
    "default": {
        "ENGINE": "django.db.backends.mysql",
        "NAME": "vaalikoppi",
        "USER": "vaalikoppi",
        "PASSWORD": "secret",
        "HOST": "db",
        "PORT": "",
        "ATOMIC_REQUESTS": True,
    }
}