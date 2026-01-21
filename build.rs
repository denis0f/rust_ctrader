fn main() -> Result<(), Box<dyn std::error::Error>>{
    tonic_prost_build::Config::new()
    .default_package_filename("open_api")
    .compile_protos(&[
        "proto/OpenApiCommonMessages.proto",
        "proto/OpenApiCommonModelMessages.proto",
        "proto/OpenApiMessages.proto",
        "proto/OpenApiModelMessages.proto"
    ], &["proto/"])?;

    Ok(())
}