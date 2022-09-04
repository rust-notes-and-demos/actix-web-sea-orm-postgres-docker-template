#!/usr/bin/env bash
set -x
set -eo pipefail

# Check if a custom user has been set, otherwise default to 'postgres'
DB_USER=${POSTGRES_USER:=postgres}

# Check if a custom password has been set, otherwise default to 'secret'
DB_PASSWORD="${POSTGRES_PASSWORD:=secret}"

# Check if a custom database name has been set, otherwise default to 'template'
DB_NAME="${POSTGRES_DB:=template}"

# Check if a custom port has been set, otherwise default to '5432'
DB_PORT="${POSTGRES_PORT:=5432}"

docker run \
  -e POSTGRES_USER=${DB_USER} \
  -e POSTGRES_PASSWORD=${DB_PASSWORD} \
  -e POSTGRES_DB=${DB_NAME} \
  -p "${DB_PORT}":5432 \
  -d postgres \
  postgres -N 1000

>&2 echo "Postgres is ready on port ${DB_PORT}!"