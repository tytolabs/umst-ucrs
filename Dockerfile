# Multi-stage build for umst-ucrs daemon (Rust/)
# SPDX-License-Identifier: MIT

FROM rust:1.88-bookworm AS builder
WORKDIR /build
COPY Rust/ ./
RUN cargo build --release --features daemon

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/umst-ucrs /usr/local/bin/umst-ucrs
EXPOSE 9090
ENV UMST_UCRS_METRICS=1
ENTRYPOINT ["umst-ucrs"]
CMD ["--tick-secs", "60"]
