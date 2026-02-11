fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../proto/kv/v1/data_service.proto"],
            &["../proto"],
        )?;
    Ok(())
}
