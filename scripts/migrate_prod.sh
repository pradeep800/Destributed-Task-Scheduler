#!/usr/bin/env bash
set -x
set -eo pipefail

run_migrations() {
  local db_user="$1"
  local db_password="$2"
  local db_name="$3"
  local db_port="$4"
  local db_host="$5"
  local migration_source="$6"
  export DATABASE_URL="postgres://${db_user}:${db_password}@${db_host}:${db_port}/${db_name}"
  sqlx database create
  sqlx migrate run --source "${migration_source}"
}

db_user=postgres
db_password=password
db_name=health_check
db_port=5432
db_host=health-checks-db-svc
run_migrations "$db_user" "$db_password" "$db_name" "$db_port" "$db_host" "crates/db/health_checks/migrations"

db_name=tasks
db_host=tasks-db-svc
run_migrations "$db_user" "$db_password" "$db_name" "$db_port" "$db_host" "crates/db/tasks/migrations"
