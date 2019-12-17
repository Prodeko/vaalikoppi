from .base import *

DEBUG = True

ALLOWED_HOSTS = ['vaalikoppi.azurewebsites.net', '127.0.0.1']

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
    }
}

 
MIDDLEWARE += ("whitenoise.middleware.WhiteNoiseMiddleware",)

STATICFILES_STORAGE = 'whitenoise.storage.CompressedManifestStaticFilesStorage'

STATIC_ROOT = os.path.join(BASE_DIR, 'staticfiles')