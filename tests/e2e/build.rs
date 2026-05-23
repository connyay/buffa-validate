fn main() {
    let validate_include = buffa_validate_build::include_dir();

    connectrpc_build::Config::new()
        .files(&["proto/test.proto"])
        .includes(&["proto/", validate_include.to_str().unwrap()])
        .include_file("_include.rs")
        .compile()
        .unwrap();

    buffa_validate_build::Config::new()
        .files(&["proto/test.proto"])
        .includes(&["proto/"])
        .compile()
        .unwrap();
}
