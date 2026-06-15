#!/usr/bin/env bash
set -euo pipefail

PG_DATA="${PGDATA:-$PWD/.dev/pg/data}"
PG_PORT="${PG_PORT:-5433}"
PG_USER="${PG_USER:-cala}"
PG_DB="${PG_DB:-cala}"

mkdir -p "$(dirname "$PG_DATA")"

if [ ! -s "$PG_DATA/PG_VERSION" ]; then
  initdb -D "$PG_DATA" -U "$PG_USER" --auth=trust --no-locale --encoding=UTF8 >/dev/null
  {
    echo "listen_addresses = '127.0.0.1'"
    echo "unix_socket_directories = '$PG_DATA'"
  } >> "$PG_DATA/postgresql.conf"
fi

(
  for _ in $(seq 1 60); do
    pg_isready -h 127.0.0.1 -p "$PG_PORT" -U "$PG_USER" >/dev/null 2>&1 && break
    sleep 0.5
  done
  if ! psql -h 127.0.0.1 -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" -c '\q' >/dev/null 2>&1; then
    createdb -h 127.0.0.1 -p "$PG_PORT" -U "$PG_USER" "$PG_DB"
  fi
) &

exec postgres -D "$PG_DATA" -p "$PG_PORT" -k "$PG_DATA"
