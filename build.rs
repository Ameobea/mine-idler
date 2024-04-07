fn main() {
  tonic_build::configure()
    .build_server(true)
    .type_attribute("ItemDescriptor", "#[derive(::serde::Deserialize)]")
    .type_attribute("ItemModifier", "#[derive(::serde::Deserialize)]")
    .type_attribute("MineLocationDescriptor", "#[derive(::serde::Deserialize)]")
    .compile(&["protos/mine.proto"], &["protos/"])
    .expect("Failed to compile protos with `prost-build`");
}
