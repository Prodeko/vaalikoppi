from django.conf.urls import include, url
from django.contrib import admin


urlpatterns = [
    url(r"^vaalikoppi/", include("vaalikoppi.urls", namespace="vaalikoppi")),
    url(r"^admin/", admin.site.urls),
]
