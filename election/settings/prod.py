from .base import *

DEBUG = False

ALLOWED_HOSTS = ["vaalikoppi.azurewebsites.net", "vaalikoppi.prodeko.org"]

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
        'OPTIONS': {
            'ssl': {'ssl-ca': os.environ.get('POSTGRESQL_SSL_CA', '')}
        },
    }
}


MIDDLEWARE += ("whitenoise.middleware.WhiteNoiseMiddleware",)

STATICFILES_STORAGE = "whitenoise.storage.CompressedManifestStaticFilesStorage"

STATIC_ROOT = os.path.join(BASE_DIR, "staticfiles")
