# Multi-stage build for w9-mail backend
# Stage 1: Build
FROM rust:1.83-slim AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY Cargo.toml Cargo.lock* ./
COPY src/ ./src/

# Build the binary
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (SQLite, wget for healthcheck)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/w9-mail-backend /app/w9-mail-backend

# Run as non-root user
RUN useradd -r -s /bin/false w9mail && \
    chown -R w9mail:w9mail /app

USER w9mail

EXPOSE 8080

ENTRYPOINT ["/app/w9-mail-backend"]

