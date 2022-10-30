from election.sentry import *

from .base import *

DEBUG = False

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
