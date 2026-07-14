# Build stage
# Pinned to the toolchain that generated the current Cargo.lock, and
# explicitly to -bookworm so it shares a glibc with the debian:bookworm-slim
# runtime image below. A bare `rust:1.96-slim` builds against Debian trixie
# (glibc 2.38+), which then fails to run at all on the older-glibc bookworm
# runtime ("GLIBC_2.38 not found") - only caught by actually running the
# container, not by `docker build` succeeding. This pin *will* still go
# stale again as transitive deps advance past this Rust version (it
# previously sat at 1.80 while the lockfile needed 1.88+, and nobody
# noticed because CI builds with dtolnay/rust-toolchain@stable, which
# floats). When bumping, verify with `docker compose up --build` end to
# end, not just `docker build` succeeding.
FROM rust:1.96-slim-bookworm as builder

# Install system dependencies needed for compiling serialport / axum / sqlx
RUN apt-get update && apt-get install -y pkg-config libssl-dev libudev-dev build-essential

# Create a new empty shell project
WORKDIR /usr/src/qsafe
COPY . .

# Build the workspace
RUN cargo build --release -p qsafe-backend

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies (OpenSSL, etc.). curl is here solely for the
# HEALTHCHECK below, which hits the existing /api/health endpoint.
RUN apt-get update && apt-get install -y ca-certificates libssl3 libudev1 curl && rm -rf /var/lib/apt/lists/*

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

# Reuses the existing /api/health endpoint rather than adding a new one.
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f "http://localhost:${PORT}/api/health" || exit 1

# Run the binary
CMD ["./qsafe-backend"]
