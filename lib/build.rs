fn main() {
    let proto_files = vec![
        "./lib/grpc/proto/rustler.proto",
        "./lib/grpc/proto/market.proto",
        "./lib/grpc/proto/ticker.proto",
    ];

    for proto_file in proto_files {
        compile_proto(proto_file);
    }
}

/// builds .proto files into `Rust` code
fn compile_proto(proto_file: &str) {
    tonic_build::configure()
        .build_server(true)
        .compile_protos(&[proto_file], &["."])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));

    println!("cargo:rerun-if-changed={}", proto_file);
}
