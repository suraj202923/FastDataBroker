# Build stage
FROM rust:latest as builder

WORKDIR /build

# Copy cargo manifest
COPY Cargo.toml .
COPY Cargo.lock .

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 fastdatabroker

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/admin-api /app/admin-api

# Create data directory
RUN mkdir -p /data && chown -R fastdatabroker:fastdatabroker /data

# Switch to non-root user
USER fastdatabroker

# Environment defaults
ENV ADMIN_API_ADDR=0.0.0.0:8080
ENV BROKER_URL=http://localhost:6000
ENV ADMIN_DB_PATH=/data/admin.db
ENV LOG_LEVEL=info
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose port
EXPOSE 8080

# Run application
ENTRYPOINT ["/app/admin-api"]
