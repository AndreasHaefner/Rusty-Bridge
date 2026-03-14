# Variablen
REGISTRY := ghcr.io/andreashaefner/rusty_bridge
TAG := latest

build-all: build-backend build-frontend

build-backend:
	docker build -t $(REGISTRY)/backend:$(TAG) -f backend/Dockerfile .

build-frontend:
	docker build -t $(REGISTRY)/frontend:$(TAG) -f frontend/Dockerfile .

push-all:
	docker push $(REGISTRY)/backend:$(TAG)
	docker push $(REGISTRY)/frontend:$(TAG)

pull-all:
	docker push $(REGISTRY)/backend:$(TAG)
	docker push $(REGISTRY)/frontend:$(TAG)

deploy:
	docker stack deploy -c rusty-bridge-deploy.yaml rust-bridge

re-deploy:
	docker stack rm rust-bridge
	docker stack deploy -c rusty-bridge-deploy.yaml rust-bridge

# Hilfsbefehl zum lokalen Testen
dev-backend:
	cd backend && cargo run

dev-frontend:
	cd frontend && trunk serve