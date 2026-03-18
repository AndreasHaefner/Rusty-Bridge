# Stage 1: Build
FROM rust:latest AS builder

# 1. Tooling installieren (WASM Target + Trunk)
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

WORKDIR /app

# 2. Nur Manifeste kopieren (Wichtig für Cache!)
COPY Cargo.toml Cargo.lock ./
COPY frontend/Cargo.toml ./frontend/
COPY shared/Cargo.toml ./shared/
# Falls das Backend im selben Workspace ist, auch dessen Toml (für die Auflösung)
COPY backend/Cargo.toml ./backend/

# 3. Dummy-Dateien für den Dependency-Build
RUN mkdir -p frontend/src shared/src && \
    echo "fn main() {}" > frontend/src/main.rs && \
    touch shared/src/lib.rs

# 4. Nur Abhängigkeiten bauen (Trunk braucht die index.html im Verzeichnis)
COPY frontend/index.html ./frontend/
WORKDIR /app/frontend
RUN trunk build --release || true

# 5. Echten Code kopieren
WORKDIR /app
COPY . .

# 6. Zeitstempel aktualisieren & Echter Build
RUN touch frontend/src/main.rs shared/src/lib.rs
WORKDIR /app/frontend
RUN trunk build --release

# Stage 2: Serve
FROM nginx:alpine
# Kopiere den 'dist' Ordner, den Trunk erzeugt hat
COPY --from=builder /app/frontend/dist /usr/share/nginx/html
# Statische Assets (Bilder, Favicon etc.) falls vorhanden
COPY ./frontend/public /usr/share/nginx/html 

