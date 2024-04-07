set dotenv-load := true

run:
  RUSTFLAGS="--cfg tokio_unstable --cfg foundations_unstable" RUST_LOG=debug cargo run -- --config=config.yml

migrate:
  sqlx migrate run

gen-protos:
  buf generate

docker-build:
  docker build -t ameo/mine-idler .
