from django.urls import path

from vaalikoppi import views

app_name = "vaalikoppi"


urlpatterns = [
    path("", views.general.index, name="index"),
    path("user/login/", views.user.user_login, name="user_login"),
    path("user/logout/", views.user.user_logout, name="user_logout"),
    path("votings/list/", views.votings.votings_list, name="votings"),
    path(
        "votings/<int:voting_id>/vote-normal/", views.votings.vote_normal, name="vote"
    ),
    path(
        "votings/<int:voting_id>/vote-ranked-choice/",
        views.votings.vote_ranked_choice,
        name="vote_ranked_choice",
    ),
    # Admin - tokens
    path("admin/tokens/", views.admin.tokens.admin_tokens, name="admin_tokens"),
    path(
        "admin/tokens/generate/",
        views.admin.tokens.generate_tokens,
        name="admin_tokens_generate",
    ),
    path("admin/tokens/print", views.admin.tokens.print_tokens, name="tokens"),
    path(
        "admin/tokens/invalidate/",
        views.admin.tokens.invalidate_token,
        name="admin_tokens_invalidate",
    ),
    path(
        "admin/tokens/invalidate/all/",
        views.admin.tokens.invalidate_all_tokens,
        name="admin_tokens_invalidate_all",
    ),
    path(
        "admin/tokens/activate/",
        views.admin.tokens.activate_token,
        name="admin_tokens_activate",
    ),
    # Admin - votings
    path("admin/votings/", views.admin.votings.admin_votings, name="admin_votings"),
    path(
        "admin/votings/create/", views.admin.votings.create_voting, name="create_voting"
    ),
    path(
        "admin/votings/<int:voting_id>/add/",
        views.admin.votings.add_candidate,
        name="add_candidate",
    ),
    path(
        "admin/votings/<int:candidate_id>/remove/",
        views.admin.votings.remove_candidate,
        name="remove_candidate",
    ),
    path(
        "admin/votings/list/",
        views.admin.votings.admin_voting_list,
        name="admin_voting_list",
    ),
    path(
        "admin/votings/results/",
        views.admin.votings.voting_results,
        name="admin_voting_results",
    ),
    path(
        "admin/votings/<int:voting_id>/open/",
        views.admin.votings.open_voting,
        name="admin_open_voting",
    ),
    path(
        "admin/votings/<int:voting_id>/close/",
        views.admin.votings.close_voting,
        name="admin_close_voting",
    ),
]
