from django.urls import include, path
from django.contrib import admin


app_name = "election"
urlpatterns = [
    path("", include("vaalikoppi.urls", namespace="vaalikoppi")),
    path("admin/", admin.site.urls),
]
