# Multi-stage production Docker build
# Optimized for minimal size, security, and fast builds with cargo-chef

# Stage 1: Build planner for cargo-chef
FROM clux/muslrust:1.88.0-stable-2025-07-27 AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS rust-planner
WORKDIR /app

# Copy workspace files
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cache dependencies
FROM chef AS rust-cacher
WORKDIR /app
ARG TARGETPLATFORM

# Copy workspace structure for dependency building
COPY --from=rust-planner /app/recipe.json recipe.json
COPY ./Cargo.toml ./Cargo.toml

RUN export TARGET_ARCH=$(case ${TARGETPLATFORM:-linux/amd64} in \
         "linux/amd64") echo "x86_64-unknown-linux-musl" ;; \
         "linux/arm64") echo "aarch64-unknown-linux-musl" ;; \
         *) echo "aarch64-unknown-linux-musl" ;; \
    esac) && \
    echo "Installing Rust target: ${TARGET_ARCH}" && \
    rustup target add ${TARGET_ARCH} && \
    echo "Cooking dependencies for target: ${TARGET_ARCH}" && \
    cargo chef cook --release --target ${TARGET_ARCH} --recipe-path recipe.json

# Stage 3: Build the application
FROM chef AS rust-builder
WORKDIR /app
ARG TARGETPLATFORM

# Copy cached dependencies and workspace structure
COPY --from=rust-cacher /app/target target
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

RUN export TARGET_ARCH=$(case ${TARGETPLATFORM:-linux/amd64} in \
         "linux/amd64") echo "x86_64-unknown-linux-musl" ;; \
         "linux/arm64") echo "aarch64-unknown-linux-musl" ;; \
         *) echo "aarch64-unknown-linux-musl" ;; \
    esac) && \
    echo "Installing Rust target: ${TARGET_ARCH}" && \
    rustup target add ${TARGET_ARCH} && \
    echo "Building binary for target: ${TARGET_ARCH}" && \
    cargo build --release --target ${TARGET_ARCH} && \
    cp target/${TARGET_ARCH}/release/aipriceaction-proxy /app/aipriceaction-proxy-bin

# Stage 4: Create the final, minimal production image
FROM alpine:3.22 AS final-image
WORKDIR /app

# Install ca-certificates for HTTPS requests
RUN apk add --no-cache ca-certificates

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