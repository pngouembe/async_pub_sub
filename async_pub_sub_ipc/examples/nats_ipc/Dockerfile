FROM lukemathwalker/cargo-chef:latest-rust-1.88 AS chef
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Copy source code and build application
COPY . .
RUN cargo build --release --example nats_ipc

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy the binary from builder stage  
COPY --from=builder /usr/src/app/target/release/examples/nats_ipc /app/nats_ipc

# Set environment variables with defaults
ENV SEND_FIRST=false
ENV NATS_URL=nats://nats:4222
ENV RUST_LOG=info

CMD ["./nats_ipc"]