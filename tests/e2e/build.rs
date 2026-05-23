fn main() {
    connectrpc_build::Config::new()
        .files(&["proto/test.proto"])
        .includes(&["proto/", "../../proto/"])
        .include_file("_include.rs")
        .compile()
        .unwrap();

    buffa_validate_build::Config::new()
        .files(&["proto/test.proto"])
        .includes(&["proto/", "../../proto/"])
        .compile()
        .unwrap();
}
