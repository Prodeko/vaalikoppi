from .base import *

DEBUG = False

ALLOWED_HOSTS = []

DATABASES = {
    "default": {
        "ENGINE": "django.db.backends.postgresql",
        "NAME": "vaalikoppi",
        "USER": DB_USER,
        "PASSWORD": DB_PSWD,
        "HOST": "prodeko-postgres.postgres.database.azure.com",
        "PORT": "5432",
        "ATOMIC_REQUESTS": True,
    }
}