FROM rust:1.75 as builder

WORKDIR /app

# Copy workspace files
COPY ../Cargo.toml ./
COPY ../common ../common
COPY . ./server

WORKDIR /app/server

# Build
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/server/target/release/schema-registry-server /usr/local/bin/

EXPOSE 3000

CMD ["schema-registry-server"]
