# Build-from-source Dockerfile
# Use this for local development and builds
# GitHub Actions uses Dockerfile.runtime with pre-built binaries for faster CI/CD

# Stage 1: Build
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release with musl target for static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# Strip binary to reduce size
RUN strip target/x86_64-unknown-linux-musl/release/whoami

# Stage 2: Runtime
FROM scratch

# Copy the binary from builder
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/whoami /whoami

# Expose port
EXPOSE 3000

# Run the binary
ENTRYPOINT ["/whoami"]
