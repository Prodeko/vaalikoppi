from django.conf.urls import url
from vaalikoppi import views

urlpatterns = [
    url(r'^$', views.index, name='index'),
    url(r'^user/status/$', views.user_status, name='user_status'),
    url(r'^user/login/$', views.user_login, name='user_login'),
    url(r'^votings/list/$', views.votings, name='votings'),
    url(r'^votings/(?P<voting_id>\d+)/vote/$', views.vote, name="vote"),
    url(r'^admin/tokens/$', views.admin_tokens, name='admin_tokens'),
    url(r'^admin/tokens/generate/$', views.generate_tokens, name='admin_tokens_generate'),
    url(r'^votings/logout/$', views.user_logout, name='user_logout'),
    url(r'^admin/tokens/invalidate/$', views.invalidate_token, name='admin_tokens_invalidate'),
    url(r'^admin/tokens/activate/$', views.activate_token, name='admin_tokens_activate'),
	url(r'^admin/votings/$', views.admin_votings, name='admin_votings'),
    url(r'^admin/votings/create/$', views.create_voting, name="create_voting"),
    url(r'^admin/votings/(?P<voting_id>\d+)/add/$', views.add_candidate, name="add_candidate"),
    url(r'^admin/votings/(?P<candidate_id>\d+)/remove/$', views.remove_candidate, name="remove_candidate"),
	url(r'^admin/votings/list/$', views.admin_voting_list, name='admin_voting_list'),
    url(r'^admin/votings/results/$', views.voting_results, name='admin_voting_results'),
	url(r'^admin/votings/(?P<voting_id>\d+)/open/$', views.open_voting, name='admin_open_voting'),
	url(r'^admin/votings/(?P<voting_id>\d+)/close/$', views.close_voting, name='admin_close_voting'),
]
