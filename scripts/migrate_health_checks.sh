#!/usr/bin/env bash
set -x
set -eo pipefail
DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=health_check}"
DB_PORT="${POSTGRES_PORT:=5433}"
DB_HOST="${POSTGRES_HOST:=localhost}"
export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run --source crates/db/health_checks/migrations 
