# Multi-stage production Docker build
# Optimized for minimal size, security, and fast builds with cargo-chef

# Stage 1: Build planner for cargo-chef
FROM rust:1.88-alpine AS chef
USER root

# Install build dependencies for Alpine
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    ca-certificates \
    gcc \
    g++ \
    make

RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS rust-planner
WORKDIR /app

# Copy only dependency files for cargo-chef
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cache dependencies
FROM chef AS rust-cacher
WORKDIR /app
ARG TARGETPLATFORM

# Copy workspace structure for dependency building
COPY --from=rust-planner /app/recipe.json recipe.json
COPY ./Cargo.toml ./Cargo.toml

# Add Alpine build dependencies for this stage
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    ca-certificates \
    gcc \
    g++ \
    make

# Use native musl target for the current architecture
RUN echo "Building for native musl target" && \
    cargo chef cook --release --recipe-path recipe.json

# Stage 3: Build the application
FROM chef AS rust-builder
WORKDIR /app
ARG TARGETPLATFORM

# Copy cached dependencies and workspace structure
COPY --from=rust-cacher /app/target target
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

# Add Alpine build dependencies for this stage
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    ca-certificates \
    gcc \
    g++ \
    make

# Build for native musl target
RUN echo "Building binary for native musl target" && \
    cargo build --release && \
    cp target/release/aipriceaction-proxy /app/aipriceaction-proxy-bin

# Stage 4: Create the final, minimal production image
FROM alpine:3.22 AS final-image
WORKDIR /app

# Install ca-certificates and curl for HTTPS requests and health checks
RUN apk add --no-cache ca-certificates curl

# Create non-root user for security
RUN addgroup -S appgroup && adduser -S -G appgroup appuser

# Set default environment variables
ENV RUST_LOG="info"

# Copy the compiled binary from rust-builder stage
COPY --from=rust-builder /app/aipriceaction-proxy-bin ./aipriceaction-proxy

# Copy ticker group configuration file
COPY ./ticker_group.json ./ticker_group.json

# Copy example configuration files (optional)
COPY ./examples/configs ./examples/configs

# Change ownership to non-root user
RUN chown -R appuser:appgroup /app

# Use non-root user
USER appuser

# Expose port (default port from main.rs seems to be configurable)
EXPOSE 8080

# Default command
CMD ["./aipriceaction-proxy"]