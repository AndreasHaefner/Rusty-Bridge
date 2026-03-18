#!/bin/bash
set -e

# Konfiguration
DB_CONTAINER_NAME="sqlx_prepare_db"
DB_USER="postgres"
DB_PASS="password"
DB_NAME="game_db"
DB_PORT="5432"

echo "Wechsle zu Remote-Kontext und deploye..."
docker context use default



#echo "🧹 Räume Docker von davor auf..."
docker stop $DB_CONTAINER_NAME
docker rm $DB_CONTAINER_NAME

echo "🚀 Starte temporären Postgres-Container..."
docker run --name $DB_CONTAINER_NAME \
  -e POSTGRES_PASSWORD=$DB_PASS \
  -e POSTGRES_DB=$DB_NAME \
  -p $DB_PORT:5432 \
  -d postgres:latest

# Datenbank-URL für SQLx
export DATABASE_URL="postgres://$DB_USER:$DB_PASS@localhost:$DB_PORT/$DB_NAME"

# Warten, bis Postgres bereit ist (Healthcheck-Loop)
echo "⏳ Warte auf Datenbank..."
until docker exec $DB_CONTAINER_NAME pg_isready -U $DB_USER; do
  sleep 1
done

echo "✅ Datenbank ist bereit. Führe Schema-Setup aus..."


for f in ./migrations/*.sql; do
    echo "Führe $f aus..."
    docker exec -i $DB_CONTAINER_NAME psql -U $DB_USER -d $DB_NAME < "$f"
done

echo "💾 Generiere SQLx Query Cache (sqlx-data.json)..."
cargo sqlx prepare

echo "🧹 Räume Docker auf..."
docker stop $DB_CONTAINER_NAME
docker rm $DB_CONTAINER_NAME

echo "🎉 Fertig! Du kannst jetzt 'SQLX_OFFLINE=true cargo build' nutzen."