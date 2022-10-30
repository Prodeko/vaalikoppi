import debug_toolbar
from django.conf import settings
from django.contrib import admin
from django.urls import include, path

from .views import redirect_view

app_name = "election"
urlpatterns = [
    path("", include("vaalikoppi.urls", namespace="vaalikoppi")),
    path("admin/", admin.site.urls),
]

if settings.SHOW_DEBUG_TOOLBAR:
    urlpatterns.append(path("__debug__/", include(debug_toolbar.urls)))

if settings.SHOW_DJANGO_SILK:
    urlpatterns.append(path("silk", include("silk.urls", namespace="silk")))
