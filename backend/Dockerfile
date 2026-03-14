# Stage 1: Build
FROM rust:latest AS builder
WORKDIR /app
COPY . .
# Baue das Backend
RUN cargo build --release --package backend

# Stage 2: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/backend /app/backend
CMD ["/app/backend"]