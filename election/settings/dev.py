import nplusone

from .base import *

# Show warnings for possible N+1 queries
# nplusone.show_nplusones()

DEBUG = True


# Set to True to enable django debug toolbar
SHOW_DEBUG_TOOLBAR = False

if SHOW_DEBUG_TOOLBAR:
    # Show django debug toolbar always.
    # This is needed because the Docker internal IP is not static
    DEBUG_TOOLBAR_CONFIG = {
        "SHOW_TOOLBAR_CALLBACK": lambda request: True if DEBUG else False
    }
    INSTALLED_APPS.append("debug_toolbar")
    MIDDLEWARE.append("debug_toolbar.middleware.DebugToolbarMiddleware")

# Set to True to enable django-silk
SHOW_DJANGO_SILK = False

if SHOW_DJANGO_SILK:
    SILKY_PYTHON_PROFILER = True
    INSTALLED_APPS.append("silk")
    MIDDLEWARE.append("silk.middleware.SilkyMiddleware")
