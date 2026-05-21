# ---- Build Frontend ----
FROM node:20-alpine AS frontend-build

WORKDIR /app/web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

# ---- Build Backend ----
FROM rust:1.82-alpine AS backend-build

RUN apk add --no-cache musl-dev pkgconf openssl-dev openssl-libs-static

WORKDIR /app/server

# Copy workspace Cargo.toml and all crate Cargo.toml files
COPY server/Cargo.toml ./
COPY server/tonevault-server/Cargo.toml tonevault-server/Cargo.toml
COPY server/tonevault-core/Cargo.toml tonevault-core/Cargo.toml
COPY server/tonevault-db/Cargo.toml tonevault-db/Cargo.toml
COPY server/tonevault-auth/Cargo.toml tonevault-auth/Cargo.toml
COPY server/tonevault-webdav/Cargo.toml tonevault-webdav/Cargo.toml

# Create dummy source files to cache dependencies
RUN mkdir -p tonevault-server/src && echo "fn main(){}" > tonevault-server/src/main.rs \
 && mkdir -p tonevault-core/src && echo "" > tonevault-core/src/lib.rs \
 && mkdir -p tonevault-db/src && echo "" > tonevault-db/src/lib.rs \
 && mkdir -p tonevault-auth/src && echo "" > tonevault-auth/src/lib.rs \
 && mkdir -p tonevault-webdav/src && echo "" > tonevault-webdav/src/lib.rs

RUN cargo build --release -p tonevault-server 2>/dev/null || true

# Copy real source code and build
COPY server/ ./
RUN touch tonevault-server/src/main.rs tonevault-core/src/lib.rs tonevault-db/src/lib.rs tonevault-auth/src/lib.rs tonevault-webdav/src/lib.rs
RUN cargo build --release -p tonevault-server

# ---- Runtime ----
FROM alpine:3.20

RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy backend binary
COPY --from=backend-build /app/server/target/release/tonevault-server /app/tonevault-server

# Copy frontend dist
COPY --from=frontend-build /app/web/dist /app/web/dist

# Copy entrypoint
COPY docker-entrypoint.sh /app/docker-entrypoint.sh
RUN chmod +x /app/docker-entrypoint.sh

# Create data directory
RUN mkdir -p /app/data /app/config

EXPOSE 3000

VOLUME ["/app/data", "/app/config", "/audiobooks"]

ENTRYPOINT ["/app/docker-entrypoint.sh"]
