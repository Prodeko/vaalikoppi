import pytest
from django.conf import settings


@pytest.fixture(autouse=True)
def enable_db_access_for_all_tests(db):
    pass


def pytest_configure(config):
    settings.NPLUSONE_RAISE = True
    # Fix for pytest-django with silk
    settings.MIDDLEWARE.remove("silk.middleware.SilkyMiddleware")
