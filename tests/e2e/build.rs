fn main() {
    buffa_validate_build::Config::new()
        .files(&["proto/test.proto"])
        .includes(&["proto/", "../../proto/"])
        .include_file("_include.rs")
        .compile()
        .unwrap();
}
