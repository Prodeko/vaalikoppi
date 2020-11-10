import debug_toolbar
from django.contrib import admin
from django.urls import include, path

from .views import redirect_view

app_name = "election"
urlpatterns = [
    path("", redirect_view),
    path("vaalikoppi/", include("vaalikoppi.urls", namespace="vaalikoppi")),
    path("admin/", admin.site.urls),
    path("silk", include("silk.urls", namespace='silk')),
    path("__debug__/", include(debug_toolbar.urls)),
]
