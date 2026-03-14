#!/bin/bash

# Variablen
REGISTRY="ghcr.io/andreashaefner/rusty_bridge"
TAG="latest"
REMOTE_CONTEXT="haefner-co.de" # Dein Name für den Docker-Context

echo "Setze Docker-Kontext auf default (lokal)..."
docker context use default

echo "Baue Images..."
docker build -t "$REGISTRY/backend:$TAG" -f backend/Dockerfile .
docker build -t "$REGISTRY/frontend:$TAG" -f frontend/Dockerfile .

echo "Pushe Images zur Registry..."
docker push "$REGISTRY/backend:$TAG"
docker push "$REGISTRY/frontend:$TAG"

echo "Wechsle zu Remote-Kontext und deploye..."
docker context use "$REMOTE_CONTEXT"

echo "Pulling latest images on remote..."
docker pull "$REGISTRY/backend:$TAG"
docker pull "$REGISTRY/frontend:$TAG"

echo "Entferne alten Stack..."
docker stack rm rust-bridge

echo "Warte 15 Sekunden auf Bereinigung..."
sleep 15

echo "Deploying stack..."
# Stelle sicher, dass die Datei lokal im aktuellen Verzeichnis liegt oder der Pfad stimmt
docker stack deploy -c ./rusty-bridge-deploy.yaml rust-bridge

echo "Wechsle zurück zu lokal..."
docker context use default

if [ $? -eq 0 ]; then
    echo "✅ Alles erledigt!"
else
    echo "❌ Fehler beim Deployment!"
    exit 1
fi