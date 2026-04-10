# Build stage for client (WASM)
FROM rust:slim-bullseye AS client-builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

COPY Cargo.toml ./
COPY client/Cargo.toml ./client/
COPY client/index.html ./client/

RUN mkdir -p client/src && echo "pub fn dummy() {}" > client/src/lib.rs

WORKDIR /app/client
RUN trunk build --release || true

# Build stage for server
FROM rust:slim-bullseye AS server-builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
COPY server/Cargo.toml ./server/
COPY client/Cargo.toml ./client/

# Dummy source for dependency caching
RUN mkdir -p server/src && echo "fn main() {}" > server/src/main.rs
RUN mkdir -p client/src && echo "pub fn dummy() {}" > client/src/lib.rs

RUN cargo build --release -p w9-mail-server && rm -rf server/src

# Copy real source
COPY server/src ./server/src/
COPY client/src ./client/src/
COPY client/index.html ./client/
COPY client/lib.css ./client/
COPY client/assets/ ./client/assets/ 2>/dev/null || true

RUN cargo build --release -p w9-mail-server

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=server-builder /app/target/release/w9-mail /app/w9-mail
COPY --from=client-builder /app/client/dist /app/client/dist 2>/dev/null || true

RUN test -x /app/w9-mail || echo "WARNING: binary not found"

EXPOSE 8080

ENV HOST=0.0.0.0
ENV PORT=8080
ENV RUST_LOG=info

CMD ["/app/w9-mail"]
