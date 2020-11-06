"""
WSGI config for prodekoorg project.

It exposes the WSGI callable as a module-level variable named ``application``.

For more information on this file, see
https://docs.djangoproject.com/en/2.2/howto/deployment/wsgi/
"""

import os
import sys
from django.core.wsgi import get_wsgi_application

root = os.path.join(os.path.dirname(__file__), "..")
sys.path.insert(0, root)
os.environ.setdefault("DJANGO_SETTINGS_MODULE", "election.settings.prod")

application = get_wsgi_application()
