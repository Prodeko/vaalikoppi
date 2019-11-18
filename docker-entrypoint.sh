#!/bin/bash

set -e

function wait_for_mysql () {
	# Check if MySQL is up and accepting connections.
	HOSTNAME=$(python <<EOF
try:
    from urllib.parse import urlparse
except ImportError:
    from urlparse import urlparse
o = urlparse('$DATABASE_URL')
print(o.hostname)
EOF
)
	until mysqladmin ping --host "$HOSTNAME" --silent; do
		>&2 echo "MySQL is unavailable - sleeping"
		sleep 1
	done
	>&2 echo "MySQL is up - continuing"
}

wait_for_mysql

# Create and run migrations
echo "Creating migrations..."
python3 manage.py makemigrations
python3 manage.py migrate

# Create a superuser for development
echo "Creating superuser..."
python manage.py shell -c "from django.contrib.auth import get_user_model; \
	User = get_user_model(); User.objects.filter(email='webbitiimi@prodeko.org').exists() or \
	User.objects.create_superuser(username='webbitiimi', password='kananugetti', email='webbitiimi@prodeko.org')"

exec "$@"