FROM python:3

ENV PYTHONUNBUFFERED 1

# Install Python and Package Libraries
RUN apt-get update && \
    apt-get install -y postgresql-client gettext

RUN apt-get update && apt-get install -y dos2unix

WORKDIR /code

COPY requirements.txt requirements.txt ./
RUN pip install -r requirements.txt

COPY . /code/
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod 777 /usr/local/bin/docker-entrypoint.sh \
    && ln -s /usr/local/bin/docker-entrypoint.sh /

ENTRYPOINT ["docker-entrypoint.sh"]

CMD ["python3", "manage.py", "runserver", "0.0.0.0:8000"]