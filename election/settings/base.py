import os

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

USER_TOKEN_VAR = "cur_token"

config = configparser.ConfigParser()
config.read(os.path.join(BASE_DIR, "prodekoorg/settings/variables.txt"))

SECRET_KEY = config["DJANGO"]["SECRET"]

DB_USER = config["DB"]["USER"]
DB_PSWD = config["DB"]["PASSWORD"]

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

STATICFILES_FINDERS = (
    "django.contrib.staticfiles.finders.FileSystemFinder",
    "django.contrib.staticfiles.finders.AppDirectoriesFinder",
)

MEDIA_ROOT = STATIC_ROOT = os.path.join(BASE_DIR, "vaalikoppi/static/media")
MEDIA_URL = "/media/"
STATIC_ROOT = STATIC_ROOT = os.path.join(BASE_DIR, "vaalikoppi/static")
STATIC_URL = "/static/"

LOGIN_URL = "/admin/"
LOGIN_REDIRECT_URL = "/vaalikoppi/admin/"
