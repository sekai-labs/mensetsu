FROM rust:1.98-slim-trixie AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    cmake \
    clang \
    build-essential \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/server/api

COPY server/api/Cargo.toml server/api/Cargo.lock server/api/build.rs ./
COPY server/api/src ./src
COPY contracts/grpc/proto /app/contracts/grpc/proto

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/server/api/target \
    cargo build --release --bin mensetsu-api && \
    cp /app/server/api/target/release/mensetsu-api /usr/local/bin/mensetsu-api

FROM debian:trixie-slim AS runner

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 appuser

WORKDIR /app
COPY --from=builder /usr/local/bin/mensetsu-api /usr/local/bin/mensetsu-api

USER appuser

ENV PORT=3000
ENV HOST=0.0.0.0

EXPOSE 3000

HEALTHCHECK --interval=5s --timeout=3s --start-period=30s --retries=6 \
  CMD curl -f http://localhost:3000/health/live || exit 1

CMD ["mensetsu-api"]
