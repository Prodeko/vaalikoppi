
# download the package lists from the repositories
apt-get update


# --- python ---

# set default python version to 3.5 (doesn't necessarily help)
ln -sf /usr/bin/python3.5 /usr/bin/python

# install pip
apt-get install -y python3-pip

# --- apache ---

# install packages
apt-get install -y apache2 libapache2-mod-wsgi

# remove default webroot
rm -rf /var/www

# symlink project as webroot
ln -fs /vagrant /var/www

# setup hosts file
VHOST=$(cat <<EOF
<VirtualHost *:80>
  DocumentRoot "/vagrant"
  ServerName localhost
  <Directory /vagrant>
    AllowOverride All
    Order Allow,Deny
    Allow From All
  </Directory>
</VirtualHost>
EOF
)
echo "${VHOST}" > /etc/apache2/sites-available/default

# enable apache rewrite module
a2enmod rewrite

# --- mysql ---

# install packages
echo mysql-server mysql-server/root_password select "vagrant" | debconf-set-selections
echo mysql-server mysql-server/root_password_again select "vagrant" | debconf-set-selections
apt-get install -y mysql-server-5.7 libmysqlclient-dev

# create database
mysql -uroot -pvagrant -e "CREATE USER 'election'@'localhost' IDENTIFIED BY 'election';"
mysql -uroot -pvagrant -e "GRANT ALL PRIVILEGES on *.* TO 'election'@'localhost';"
mysql -uroot -pvagrant -e "CREATE DATABASE election;"


# --- Required python modules ---
pip3 install -r /vagrant/requirements.txt

# tasks
cd /vagrant && python3 manage.py syncdb --noinput
cd /vagrant && python3 manage.py migrate

# Run server and static file watcher in screen
su - ubuntu -c "cd /vagrant && screen -S server -d -m python3 manage.py runserver 0.0.0.0:8000"
su - ubuntu -c "cd /vagrant && screen -S watcher -d -m python3 manage.py watchstatic"

# --- restart apache ---

service apache2 restart
