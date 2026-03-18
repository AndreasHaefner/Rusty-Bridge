# Stage 1: Build
FROM rust:latest AS builder
WORKDIR /app

# 1. Manifeste kopieren (ALLES was im Workspace ist)
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml ./backend/
COPY frontend/Cargo.toml ./frontend/
COPY shared/Cargo.toml ./shared/
# HIER FEHLTE ES:
COPY tests/Cargo.toml ./tests/ 

# 2. Dummy-Source-Dateien erstellen
# Wir müssen für JEDES Member eine Datei erstellen
RUN mkdir -p backend/src frontend/src shared/src tests/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    echo "fn main() {}" > frontend/src/main.rs && \
    touch shared/src/lib.rs && \
    echo "fn main() {}" > tests/src/main.rs 
    # ^ Falls tests eine Library ist, nimm touch tests/src/lib.rs

# 3. Jetzt klappt der Vorkompilier-Schritt
RUN cargo build --release --package backend

# 4. Echten Code kopieren
# Jetzt überschreiben wir die Dummys mit deinem echten Code
COPY . .

# 5. Den Zeitstempel der Dateien aktualisieren (WICHTIG!)
# Da die Cargo.toml sich nicht geändert hat, merkt Cargo sonst nicht, 
# dass der Source-Code jetzt neu ist.
RUN touch backend/src/main.rs shared/src/lib.rs

# 6. Der echte Build
# Das dauert jetzt nur noch Sekunden, da 95% schon in Schritt 3 gebaut wurden
RUN cargo build --release --package backend

# Stage 2: Runtime
FROM debian:bookworm-slim
# SSL ist wichtig für reqwest/HTTPS
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/backend ./backend

# Falls du statische Dateien (Frontend-Build) im Backend auslieferst:
# COPY --from=frontend_builder /app/frontend/dist ./dist

CMD ["./backend"]