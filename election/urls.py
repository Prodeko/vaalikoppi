from django.urls import include, path
from django.contrib import admin
from .views import redirect_view


app_name = "election"
urlpatterns = [
    path("", redirect_view),
    path("vaalikoppi/", include("vaalikoppi.urls", namespace="vaalikoppi")),
    path("admin/", admin.site.urls),
]
