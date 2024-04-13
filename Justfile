set dotenv-load := true

run:
  RUSTFLAGS="--cfg tokio_unstable --cfg foundations_unstable" RUST_LOG=debug cargo run -- --config=config.yml

migrate:
  sqlx migrate run

gen-protos:
  buf generate

docker-build:
  docker build --network host -t ameo/mine-idler .

build-and-deploy:
  just docker-build
  docker save ameo/mine-idler:latest| bzip2 > /tmp/idler.tar.bz2
  scp /tmp/idler.tar.bz2 debian@ameo.dev:/tmp/idler.tar.bz2
  ssh debian@ameo.dev -t 'cat /tmp/idler.tar.bz2 | bunzip2 | docker load && docker kill mine-idler-server  && docker container rm mine-idler-server && docker run   --name mine-idler-server   --restart=always   -d   -p 5900:5900   -p 5901:5901   -v /opt/conf/mine-idler/config.yml:/opt/config.yml   -e RUST_LOG=info   ameo/mine-idler:latest   /usr/local/bin/mine-idler --config /opt/config.yml && rm /tmp/idler.tar.bz2'

  cd frontend && just build && just deploy
