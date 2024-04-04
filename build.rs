fn main() {
  tonic_build::configure()
    .build_server(true)
    .compile(&["protos/mine.proto"], &["protos/"])
    .expect("Failed to compile protos with `prost-build`");
}
