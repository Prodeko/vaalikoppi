from django.conf.urls import url
from . import views

urlpatterns = [
    url(r'^$', views.index, name='index'),
    url(r'^votings/$', views.votings, name='votings'),
    url(r'^user/status/$', views.user_status, name='user_status'),
    url(r'^user/login/$', views.user_login, name='user_login'),
    url(r'^admin/tokens/$', views.admin_tokens, name='admin_tokens'),
    url(r'^admin/tokens/generate/$', views.generate_tokens, name='admin_tokens_generate'),
    url(r'^admin/tokens/invalidate/$', views.invalidate_token, name='admin_tokens_invalidate'),
    url(r'^(?P<voting_id>\d+)/results/$', views.results, name="results"),
    url(r'^(?P<voting_id>\d+)/vote/$', views.vote, name="vote"),
]
