FROM rust:1.40.0-stretch as cargo-build

RUN apt-get update && apt-get install -y --no-install-recommends \
 libxcb-shape0-dev=1.12-1 libxcb-xfixes0-dev=1.12-1 \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app/

COPY Cargo.toml Cargo.toml

RUN mkdir src/

RUN echo "fn main() {println!(\"failed to build\")}" > src/main.rs

RUN cargo build --release

RUN rm -f target/release/deps/myapp*

COPY . .

RUN cargo build --release

RUN mkdir -p build-out

RUN cp target/release/kmon build-out/

FROM debian:stretch-slim as runtime-image

RUN apt-get update && apt-get install -y --no-install-recommends \
 libxcb-shape0-dev=1.12-1 libxcb-xfixes0-dev=1.12-1 \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

COPY --from=cargo-build /app/build-out/kmon .

CMD ["./kmon"]