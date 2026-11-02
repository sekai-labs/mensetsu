fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
    let proto_dir = "../../contracts/grpc/proto/v1";
    println!("cargo:rerun-if-changed={proto_dir}/control.proto");
    println!("cargo:rerun-if-changed={proto_dir}/streaming.proto");
    println!("cargo:rerun-if-changed={proto_dir}/health.proto");
    println!("cargo:rerun-if-changed={proto_dir}/resume_storage.proto");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                format!("{proto_dir}/health.proto"),
                format!("{proto_dir}/control.proto"),
                format!("{proto_dir}/streaming.proto"),
                format!("{proto_dir}/resume_storage.proto"),
            ],
            &[proto_dir],
        )?;

    Ok(())
}
