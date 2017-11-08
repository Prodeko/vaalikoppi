from django.conf.urls import url
from . import views

urlpatterns = [
    url(r'^$', views.index, name='index'),
    url(r'^votings/$', views.votings, name='votings'),
    url(r'^(?P<voting_id>\d+)/results/$', views.results, name="results"),
    url(r'^(?P<voting_id>\d+)/vote/$', views.vote, name="vote"),
]
