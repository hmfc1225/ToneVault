# Stage 1: Build frontend
FROM node:20-alpine AS frontend
WORKDIR /app/web
COPY web/package*.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

# Stage 2: Build backend
FROM rust:1.82-bookworm AS backend
WORKDIR /app
COPY server/ ./server/
WORKDIR /app/server
RUN cargo build --release --features sqlite

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libsqlite3-0 && rm -rf /var/lib/apt/lists/*
COPY --from=backend /app/server/target/release/tonevault-server /usr/local/bin/
COPY --from=frontend /app/web/dist /app/web/dist
COPY config/tonevault.example.toml /app/config/tonevault.toml
WORKDIR /app
EXPOSE 8080
CMD ["tonevault-server"]
