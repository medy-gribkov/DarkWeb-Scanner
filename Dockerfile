# Multi-stage Dockerfile for sporesec-darkweb-scanner
# Build stage
FROM rust:latest AS builder
WORKDIR /usr/src/sporesec
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY Cargo.* ./
COPY src ./src
RUN cargo build --release --quiet 2>&1 | grep -v "Compiling\|Finished" || true

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/sporesec/target/release/sporesec-darkweb-scanner /usr/local/bin/sporesec
EXPOSE 8080
CMD ["sporesec"]
