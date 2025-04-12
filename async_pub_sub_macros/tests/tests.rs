#[test]
fn expand() {
    macrotest::expand("tests/expand/*.rs");
}

// Test compilation failures
#[test]
fn test_compile_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/*.rs");
}

#[test]
fn test_successful_compilation() {
    let t = trybuild::TestCases::new();
    std::fs::read_dir("tests/expand")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            !entry
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("expanded.rs"))
        })
        .for_each(|entry| t.pass(entry.path()));
}
