#!/usr/bin/env bash

CONTAINER_NAME="z2p-db"

set -uexo pipefail

if ! [ -x  "$(which sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed"
  echo >&2 "Use:"
  echo >&2 "    cargo install sqlx-cli --no-default-features --features rustls,postgres"
  echo >&2 "to install it"
  exit 1
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=Password@123}"
DB_NAME="${POSTGRESS_DB:=z2p}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

if ! [ $(docker ps -f "name"="${CONTAINER_NAME}" --format '{{.Names}}') == "${CONTAINER_NAME}" ]; then
  docker run --name z2p-db \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRESS_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    -d postgres \
    postgres -N 1000
fi

until docker exec z2p-db psql -U "${DB_USER}" -W "${DB_USER}" -c '\q'; do
  >&2 echo "\nPostgres is still unavailable -- sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running at ${DB_HOST}:${DB_PORT}"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run

>&2 echo "Database has been migrated, ready to go"