# Builder
FROM rust:1.56-slim-buster as builder
RUN apt-get update && apt-get install -y --no-install-recommends \
  libxcb1-dev=1.13.1-2 libxcb-shape0-dev=1.13.1-2 libxcb-xfixes0-dev=1.13.1-2 \
  python3 --allow-unauthenticated \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app/
COPY Cargo.toml Cargo.toml
RUN mkdir src/ && echo "fn main() {println!(\"failed to build\")}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/kmon*
COPY . .
RUN cargo build --release
RUN mkdir -p build-out && cp target/release/kmon build-out/

# Runtime
FROM debian:buster-slim as runtime-image
RUN apt-get update && apt-get install -y --no-install-recommends \
  libxcb1-dev=1.13.1-2 libxcb-shape0-dev=1.13.1-2 libxcb-xfixes0-dev=1.13.1-2 \
  kmod=26-1 --allow-unauthenticated \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /root/
COPY --from=builder /app/build-out/kmon .
CMD ["./kmon"]
