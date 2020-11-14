import pytest
from cacheops import invalidate_all


@pytest.fixture(autouse=True)
def enable_db_access_for_all_tests(db):
    pass


@pytest.fixture(autouse=True)
def django_cache():
    # Invalidate cache between test runs for consistency
    invalidate_all()
