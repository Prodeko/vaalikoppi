import json

import pytest
from django.conf import settings
from django.test import override_settings


def test_number_of_sql_queries_index(client, django_assert_num_queries):
    with django_assert_num_queries(6):

        res = client.get("/vaalikoppi/")

        assert res.status_code == 200
