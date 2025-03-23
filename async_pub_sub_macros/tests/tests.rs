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
