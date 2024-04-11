# Builder
FROM rust:1.77.2-slim-buster as builder
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    --allow-unauthenticated \
    libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev \
    && apt-get clean && rm -rf /var/lib/apt/lists/*
WORKDIR /app/
COPY Cargo.toml Cargo.toml
RUN mkdir src/ && echo "fn main() {println!(\"failed to build\")}" > src/main.rs
RUN cargo build --release --verbose
RUN rm -f target/release/deps/kmon*
COPY . .
RUN cargo build --locked --release --verbose
RUN mkdir -p build-out && cp target/release/kmon build-out/

# Runtime
FROM debian:buster-slim as runtime-image
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    --allow-unauthenticated \
    kmod \
    libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev \
    && apt-get clean && rm -rf /var/lib/apt/lists/*
WORKDIR /root/
COPY --from=builder /app/build-out/kmon .
CMD ["./kmon"]
