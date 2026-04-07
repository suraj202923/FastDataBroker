# Multi-stage build for FastDataBroker
FROM rust:nightly-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Build release binary
RUN cargo build --release --bin FastDataBroker-cli

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create appuser
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/FastDataBroker-cli /app/FastDataBroker-cli

# Set permissions for binary
RUN chmod +x /app/FastDataBroker-cli

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:6379/health || exit 1

# Switch to non-root user
USER appuser

# Expose ports
EXPOSE 6379 6380 6381

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV FastDataBroker_BIND=0.0.0.0:6379

# Run the service
CMD ["/app/FastDataBroker-cli", "server"]
