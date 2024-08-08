#!/usr/bin/env bash
set -x
set -eo pipefail

run_migrations() {
  local db_user="$1"
  local db_password="$2"
  local db_name="$3"
  local db_port="$4"
  local db_host="$5"
  export DATABASE_URL="postgres://${db_user}:${db_password}@${db_host}:${db_port}/${db_name}"
  sqlx database create
  sqlx migrate run --source crates/db/migrations
}

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=health_check}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:="health-checks-db-svc"}"
run_migrations "$DB_USER" "$DB_PASSWORD" "$DB_NAME" "$DB_PORT" "$DB_HOST"

DB_NAME="${POSTGRES_DB:=tasks}"
DB_HOST="${POSTGRES_HOST:="tasks-db-svc"}"
run_migrations "$DB_USER" "$DB_PASSWORD" "$DB_NAME" "$DB_PORT" "$DB_HOST"
