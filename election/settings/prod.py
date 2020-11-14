from election.sentry import *

from .base import *

DEBUG = False

ALLOWED_HOSTS = ["vaalikoppi.prodeko.org", "0.0.0.0"]

DB_USER = os.environ.get("DB_USER")
DB_PASSWORD = os.environ.get("DB_PASSWORD")

DATABASES = {
    "default": {
        "ENGINE": "django.db.backends.postgresql",
        "NAME": "vaalikoppi",
        "USER": DB_USER,
        "PASSWORD": DB_PASSWORD,
        "HOST": "prodeko-postgres.postgres.database.azure.com",
        "PORT": "5432",
        "ATOMIC_REQUESTS": True,
        "OPTIONS": {
            "sslmode": "verify-ca",
            "sslrootcert": os.environ.get("POSTGRESQL_SSL_CA", ""),
        },
    }
}

LOGGING = {
    "version": 1,
    "disable_existing_loggers": False,
    "filters": {"require_debug_false": {"()": "django.utils.log.RequireDebugFalse"}},
    "handlers": {
        "logfile": {
            "class": "logging.handlers.WatchedFileHandler",
            "filename": "/code/vaalikoppi.log",
        }
    },
    "loggers": {
        "django": {
            "handlers": ["logfile"],
            "level": "ERROR",
            "propagate": False,
        }
    },
}

# Don't show debg toolbar or sil in production
SHOW_DEBUG_TOOLBAR = False
SHOW_DJANGO_SILK = False
