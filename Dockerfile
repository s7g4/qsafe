# Build stage
FROM rust:1.80-slim as builder

# Install system dependencies needed for compiling serialport / axum / sqlx
RUN apt-get update && apt-get install -y pkg-config libssl-dev libudev-dev build-essential

# Create a new empty shell project
WORKDIR /usr/src/qsafe
COPY . .

# Build the workspace
RUN cargo build --release -p qsafe-backend

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies (OpenSSL, etc.)
RUN apt-get update && apt-get install -y ca-certificates libssl3 libudev1 && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -ms /bin/bash qsafe
USER qsafe
WORKDIR /app

# Copy the compiled binary from the builder environment
COPY --from=builder /usr/src/qsafe/target/release/qsafe-backend /app/qsafe-backend

# Expose API port
EXPOSE 3000

# Set environment variables with defaults (should be overridden by docker-compose)
ENV PORT=3000
ENV HOST=0.0.0.0
ENV HSM_MOCK=true

# Run the binary
CMD ["./qsafe-backend"]
