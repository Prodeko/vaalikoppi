import os

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

USER_TOKEN_VAR = "cur_token"

# SECURITY WARNING: keep the secret key used in production secret!
SECRET_KEY = "wavy&g8u^zge&8!_$%vz5g@&y-wi%&yrzvzlz33pv&+t(svrmr"

# SECURITY WARNING: don't run with debug turned on in production!
DEBUG = True
ALLOWED_HOSTS = []

INSTALLED_APPS = [
    "vaalikoppi.apps.VaalikoppiConfig",
    "django.contrib.admin",
    "django.contrib.auth",
    "django.contrib.contenttypes",
    "django.contrib.sessions",
    "django.contrib.messages",
    "django.contrib.staticfiles",
]

MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "django.contrib.sessions.middleware.SessionMiddleware",
    "django.middleware.common.CommonMiddleware",
    "django.middleware.csrf.CsrfViewMiddleware",
    "django.contrib.auth.middleware.AuthenticationMiddleware",
    "django.contrib.sessions.middleware.SessionMiddleware",
    "django.contrib.messages.middleware.MessageMiddleware",
    "django.middleware.clickjacking.XFrameOptionsMiddleware",
]

ROOT_URLCONF = "election.urls"

TEMPLATES = [
    {
        "BACKEND": "django.template.backends.django.DjangoTemplates",
        "DIRS": [],
        "APP_DIRS": True,
        "OPTIONS": {
            "context_processors": [
                "django.template.context_processors.debug",
                "django.template.context_processors.request",
                "django.contrib.auth.context_processors.auth",
                "django.contrib.messages.context_processors.messages",
            ]
        },
    }
]

WSGI_APPLICATION = "election.wsgi.application"

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

AUTH_PASSWORD_VALIDATORS = [
    {
        "NAME": "django.contrib.auth.password_validation.UserAttributeSimilarityValidator"
    },
    {"NAME": "django.contrib.auth.password_validation.MinimumLengthValidator"},
    {"NAME": "django.contrib.auth.password_validation.CommonPasswordValidator"},
    {"NAME": "django.contrib.auth.password_validation.NumericPasswordValidator"},
]

LANGUAGE_CODE = "en-us"
TIME_ZONE = "UTC"
USE_I18N = True
USE_L10N = True
USE_TZ = True

MEDIA_ROOT = "/vaalikoppi/static/media"
MEDIA_URL = "/media/"
STATIC_ROOT = "/vaalikoppi/static/"
STATIC_URL = "/static/"

LOGIN_URL = "/admin/"
LOGIN_REDIRECT_URL = "/vaalikoppi/admin/"
