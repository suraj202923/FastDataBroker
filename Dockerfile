# Multi-stage build for FastDataBroker
FROM rust:latest AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Build release binary
RUN cargo build --release

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

# Copy library from builder
COPY --from=builder /app/target/release/ /app/lib/

# Set permissions for libraries
RUN chmod -R +x /app/lib/

# Health check - verify library exists
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD test -f /app/lib/libfastdatabroker.so || exit 1

# Switch to non-root user
USER appuser

# Expose ports
EXPOSE 6379 6380 6381

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV FastDataBroker_BIND=0.0.0.0:6379

# Run the service (library container - keeps running)
CMD ["/bin/bash"]
