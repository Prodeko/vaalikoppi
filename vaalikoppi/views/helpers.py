import re

from django.conf import settings
from django.shortcuts import render
from vaalikoppi.exceptions import AliasException
from vaalikoppi.models import *


def validate_token(func):
    def wrapper(request, *args, **kwargs):
        token = get_token_from_session(request)
        if is_valid_token(token):
            return func(request, token=token, *args, **kwargs)
        return render(request, "voting-list-error.html", {"token": token})

    return wrapper


def is_valid_token(token):
    if token is not None and token.activated and not token.invalidated:
        return True

    return False


def get_token_from_session(request):
    session_var_name = settings.USER_TOKEN_VAR

    if session_var_name in request.session:
        session_token = request.session[session_var_name]

        try:
            return Usertoken.objects.get(token=session_token)
        except Usertoken.DoesNotExist:
            return None

    return None


def is_eligible_to_vote_normal(token, voting):
    #token_mapping = next(t for t in voting.token_mappings.all() if t.token == token)

    token_mappings = [t for t in voting.token_mappings.all() if t.token == token]
    if len(token_mappings) == 1:
        token_mapping = token_mappings[0]

    if not token_mapping:
        return False

    votes_count = len([v for v in voting.votes.all() if v.uuid == token_mapping.uuid])
    # User hasn't voted yet, ok to vote
    if votes_count == 0:
        return True

    # Strict policy: don't let the user vote even in a case where
    # 0 < len(votes_count) < max_votes. Should never happen.
    return False


def is_eligible_to_vote_ranked_choice(token, voting):
    #token_mapping = next(t for t in voting.token_mappings.all() if t.token == token)

    token_mappings = [t for t in voting.token_mappings.all() if t.token == token]
    if len(token_mappings) == 1:
        token_mapping = token_mappings[0]

    if not token_mapping:
        return False

    votes_by_token_count = len(
        [v for v in voting.votes.all() if v.uuid == token_mapping.uuid]
    )
    candidates_in_election_count = voting.candidates.all().count()

    if votes_by_token_count == candidates_in_election_count:
        return False

    votegroups_by_token = len(
        [v for v in voting.votegroups.all() if v.uuid == token_mapping.uuid]
    )

    # User hasn't voted yet, ok to vote
    if votegroups_by_token == 0:
        return True

    # Strict policy: don't let the user vote even in a case where
    # 0 < len(cur_votes) < max_votes. Should never happen.
    return False


def votings_list_data(request, token, is_admin=False):
    # Prefetch related for 'voter_statuses' is only used in admin view
    args_default = [
        "candidates",
        "voting_results",
        "token_mappings",
        "token_mappings__token",
        "votes",
    ]
    args_ranked_choice = [
        "votegroups",
    ]
    args_admin = ["voter_statuses"]

    args = args_default
    if is_admin:
        args += args_admin

    v1 = list(
        RankedChoiceVoting.objects.prefetch_related(*(args + args_ranked_choice)).all()
    )

    v2 = list(NormalVoting.objects.prefetch_related(*(args)).all())

    votings = v1 + v2
    votings.sort(key=lambda x: x.created_at, reverse=True)

    open_votings = []
    closed_votings = []
    ended_votings = []

    for v in votings:
        # Set is_eligible so that all votings can be seen on admin view
        if is_admin:
            is_eligible = True
        elif token is None or not v.is_open or v.is_ended:
            is_eligible = False
        elif v.is_ranked_choice:
            is_eligible = is_eligible_to_vote_ranked_choice(token, v)
        elif not v.is_ranked_choice:
            is_eligible = is_eligible_to_vote_normal(token, v)
        else:
            # ... should never end up here
            is_eligible = False

        if v.is_open and is_eligible:
            # Voting is open and user is eligible to vote
            open_votings.append(v)
        elif v.is_open and not is_eligible:
            # Voting is open but user is not eligible to vote
            closed_votings.append(v)

        if not v.is_open and not v.is_ended:
            # Voting is closed
            closed_votings.append(v)

        if not v.is_open and v.is_ended:
            # Voting is ended
            ended_votings.append(v)

        # Enhance voting objects for admin view
        if is_admin:
            v.tokens_not_voted = []
            mappings = v.token_mappings.all()

            for m in mappings:
                votes_count = len([v for v in v.votes.all() if v.uuid == m.uuid])

                if votes_count == 0:
                    v.tokens_not_voted.append(m.token)

    return {
        "is_admin": is_admin,
        "open_votings": open_votings,
        "closed_votings": closed_votings,
        "ended_votings": ended_votings,
    }


def validate_register_alias(request, token_obj, alias):
    alias_regex = (
        r"^[A-Z0-9\u00C0-\u00D6\u00D8-\u00DE][A-Z0-9\u00C0-\u00D6\u00D8-\u00DE_\-]+$"
    )
    alias = alias.upper()
    alias_len = len(alias)
    active_token = Usertoken.objects.filter(activated=True, invalidated=False)

    # Allow an existing alias to be used if registered for the current token
    if (
        alias_len >= 3
        and alias_len <= 20
        and bool(re.match(alias_regex, alias))
        and 0 == active_token.exclude(token=token_obj.token).filter(alias=alias).count()
    ):
        token = Usertoken.objects.get(token=token_obj.token)
        token.alias = alias
        token.save()
    else:
        raise AliasException("Invalid alias provided")
