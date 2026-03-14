# Stage 1: Build
FROM rust:latest AS builder
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk
WORKDIR /app
COPY . .
WORKDIR /app/frontend
RUN trunk build --release

# Stage 2: Serve
FROM nginx:alpine
COPY --from=builder /app/frontend/dist /usr/share/nginx/html
COPY ./frontend/public /usr/share/nginx/html
