# Mine Idler

WIP browser-based idle game

## Local Dev Setup

Generate default config with `RUSTFLAGS="--cfg tokio_unstable --cfg foundations_unstable" RUST_LOG=debug cargo run -- -g ./default-config.yml`

You'll need a Postgres server.  Put config in `config.yml.

Run with `just run`.

Test gRPC endpoints `grpcurl` like this:

`grpcurl -plaintext -import-path <...>/mine-idler/protos -proto mine.proto -H 'authorization: <session_token>' localhost:5900 mine.MineService.StartMining`
