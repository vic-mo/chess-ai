# Multi-stage build for Rust WebSocket server
# Optimized for Google Cloud Run deployment

# Stage 1: Build stage
FROM rust:1.82-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY apps/uci-server/ ./apps/uci-server/

# Build in release mode
RUN cargo build --release --bin uci-server

# Stage 2: Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/uci-server /app/uci-server

# Cloud Run requires the container to listen on the PORT environment variable
ENV PORT=8080

# Expose the port (documentation only, Cloud Run ignores this)
EXPOSE 8080

# Run the binary
CMD ["/app/uci-server"]
