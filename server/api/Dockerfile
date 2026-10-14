FROM rust:1.98-slim-trixie AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    cmake \
    clang \
    build-essential \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

ARG CARGO_BUILD_JOBS=2
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/mensetsu_api* target/release/mensetsu-api* target/release/.fingerprint/mensetsu-api-*

COPY src ./src

RUN cargo build --release --bin mensetsu-api

FROM debian:trixie-slim AS runner

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 appuser

WORKDIR /app
COPY --from=builder /app/target/release/mensetsu-api /usr/local/bin/mensetsu-api

USER appuser

ENV PORT=3000
ENV HOST=0.0.0.0

EXPOSE 3000

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health/live || exit 1

CMD ["mensetsu-api"]
