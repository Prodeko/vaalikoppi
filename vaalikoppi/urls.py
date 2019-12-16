from django.urls import path
from vaalikoppi import views

app_name = "vaalikoppi"

urlpatterns = [
    path("", views.index, name="index"),
    path("user/login/", views.user_login, name="user_login"),
    path("user/logout/", views.user_logout, name="user_logout"),
    path("user/status/", views.user_status, name="user_status"),
    path("votings/list/", views.votings, name="votings"),
    path("votings/<int:voting_id>/vote/", views.vote, name="vote"),
    path("votings/<int:voting_id>/voteTransferable/", views.vote_transferable, name="vote_transferable"),
    path("admin/tokens/", views.admin_tokens, name="admin_tokens"),
    path("admin/tokens/generate/", views.generate_tokens, name="admin_tokens_generate"),
    path("admin/tokens/print", views.tokens, name="tokens"),
    path("admin/tokens/invalidate/", views.invalidate_token, name="admin_tokens_invalidate"),
    path("admin/tokens/invalidate/all/", views.invalidate_all_tokens, name="admin_tokens_invalidate_all"),
    path("admin/tokens/activate/", views.activate_token, name="admin_tokens_activate"),
	path("admin/votings/", views.admin_votings, name="admin_votings"),
    path("admin/votings/create/", views.create_voting, name="create_voting"),
    path("admin/votings/<int:voting_id>/add/", views.add_candidate, name="add_candidate"),
    path("admin/votings/<int:candidate_id>/remove/", views.remove_candidate, name="remove_candidate"),
	path("admin/votings/list/", views.admin_voting_list, name="admin_voting_list"),
    path("admin/votings/results/", views.voting_results, name="admin_voting_results"),
	path("admin/votings/<int:voting_id>/open/", views.open_voting, name="admin_open_voting"),
	path("admin/votings/<int:voting_id>/close/", views.close_voting, name="admin_close_voting"),
    path("admin/test/<int:voting_id>/", views.test, name="test"),
]
