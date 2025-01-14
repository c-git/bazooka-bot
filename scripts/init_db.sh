#!/usr/bin/env bash
# set -x # Only turn on if debugging otherwise it just seems annoying
set -eo pipefail

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install --version='~0.8' sqlx-cli --no-default-features --features rustls,postgres"
  echo >&2 "to install it."
  exit 1
fi

# Check if a custom parameter has been set, otherwise use default values
DB_PORT="${DB_PORT:=5432}"
SUPERUSER="${SUPERUSER:=postgres}"
SUPERUSER_PWD="${SUPERUSER_PWD:=password}"
APP_USER="${APP_USER:=db_user}"
APP_USER_PWD="${APP_USER_PWD:=password}"
APP_DB_NAME="${APP_DB_NAME:=bazooka_bot}"

# Allow to skip Docker if a Postgres database is already running
if [[ -z "${SKIP_DOCKER}" ]]
then
  # if a postgres container is running, print instructions to kill it and exit
  RUNNING_POSTGRES_CONTAINER=$(docker ps --filter 'name=postgres' --format '{{.ID}}')
  if [[ -n $RUNNING_POSTGRES_CONTAINER ]]; then
    echo >&2 "there is a postgres container already running, kill it with"
    echo >&2 "    docker kill ${RUNNING_POSTGRES_CONTAINER}"
    exit 1
  fi
  CONTAINER_NAME="postgres_$(date '+%s')"
  # Launch postgres using Docker
  echo "Container ID is:"
  docker run \
      --env POSTGRES_USER=${SUPERUSER} \
      --env POSTGRES_PASSWORD=${SUPERUSER_PWD} \
      --health-cmd="pg_isready -U ${SUPERUSER} || exit 1" \
      --health-interval=1s \
      --health-timeout=5s \
      --health-retries=5 \
      --publish "${DB_PORT}":5432 \
      --detach \
      --name "${CONTAINER_NAME}" \
      postgres -N 1000
      # ^ Increased maximum number of connections for testing purposes

  max_retries=120
  until [ \
    "$(docker inspect -f "{{.State.Health.Status}}" ${CONTAINER_NAME})" == \
    "healthy" \
  ]; do     
    >&2 echo "Postgres is still unavailable - sleeping ($max_retries attempts left)"
    if [ $max_retries -lt 1 ];  then
        >&2 echo "Exceeded attempts to connect to DB"
        exit 1
    fi
    sleep 1
    ((max_retries--))
  done
  
  # Create the application user
  CREATE_QUERY="CREATE USER ${APP_USER} WITH PASSWORD '${APP_USER_PWD}';"
  docker exec -it "${CONTAINER_NAME}" psql -U "${SUPERUSER}" -c "${CREATE_QUERY}"
  
  # Grant create db privileges to the app user
  GRANT_QUERY="ALTER USER ${APP_USER} CREATEDB;"
  docker exec -it "${CONTAINER_NAME}" psql -U "${SUPERUSER}" -c "${GRANT_QUERY}"
fi

>&2 echo "Postgres is up and running on port ${DB_PORT} - running migrations now!"

# Create the application database
export DATABASE_URL=postgres://${APP_USER}:${APP_USER_PWD}@localhost:${DB_PORT}/${APP_DB_NAME}
sqlx database create
sqlx migrate run --source migrations_pg

>&2 echo "Postgres has been migrated, ready to go!"