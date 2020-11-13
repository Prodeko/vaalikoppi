import pytest
from cacheops import invalidate_all
from django.conf import settings
from django.core.cache import cache
from pytest_django.lazy_django import skip_if_no_django


@pytest.fixture(autouse=True)
def enable_db_access_for_all_tests(db):
    pass

@pytest.fixture(autouse=True)
def django_cache():
    # Invalidate cache between test runss for consistency
    invalidate_all()
