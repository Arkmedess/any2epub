# Multi-stage Dockerfile para any2epub
FROM rust:1.82-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Pre-build dummy to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "pub fn dummy() {}" > src/lib.rs && \
    mkdir -p src/bin && echo "fn main() {}" > src/bin/pdf2epub.rs && \
    cargo build --release || true && \
    rm -rf src

# Copia código-fonte real
COPY src ./src
COPY tests ./tests
COPY docs ./docs
COPY README.md ./

# Compilação otimizada em release
RUN cargo build --release

# Estágio final enxuto e seguro (rootless)
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Cria usuário não-root
RUN useradd -m -u 10001 appuser
USER appuser
WORKDIR /home/appuser

COPY --from=builder --chown=appuser:appuser /app/target/release/any2epub /usr/local/bin/any2epub
COPY --from=builder --chown=appuser:appuser /app/target/release/pdf2epub /usr/local/bin/pdf2epub

ENTRYPOINT ["any2epub"]
CMD ["--help"]
