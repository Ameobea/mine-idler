FROM debian:bookworm-slim AS builder

RUN apt-get update && apt-get install -y curl build-essential libssl-dev pkg-config protobuf-compiler libpq-dev
RUN update-ca-certificates

# Install rust
RUN curl https://sh.rustup.rs/ -sSf | \
  sh -s -- -y --default-toolchain nightly-2024-02-03

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
ADD . ./

RUN RUSTFLAGS="--cfg tokio_unstable --cfg foundations_unstable" cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libpq-dev

COPY --from=builder \
  /app/target/release/mine-idler \
  /usr/local/bin/

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && update-ca-certificates
WORKDIR /root
RUN touch .env
CMD /usr/local/bin/mine-idler
