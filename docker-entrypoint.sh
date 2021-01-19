#!/bin/bash

set -e

timer="2"

function wait_for_psql () {
    until pg_isready --username=vaalikoppi --host=db 2>/dev/null; do
    >&2 echo "Postgres is unavailable - sleeping for $timer seconds"
    sleep $timer
    done

    >&2 echo "Postgres is up - executing command"
}

wait_for_psql

# Create and run migrations
echo "Creating migrations..."
python3 manage.py makemigrations
python3 manage.py migrate

echo "Collecting static files..."
python3 manage.py collectstatic

# Create a superuser for development
echo "Creating superuser..."
python manage.py shell -c "from django.contrib.auth import get_user_model; \
	User = get_user_model(); User.objects.filter(email='webbitiimi@prodeko.org').exists() or \
	User.objects.create_superuser(username='webbitiimi', password='kananugetti', email='webbitiimi@prodeko.org')"

nohup python manage.py sass vaalikoppi/static/scss vaalikoppi/static/css --watch &

exec "$@"