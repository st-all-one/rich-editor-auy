//! Compile-fail contract for the declarative schema front-end.

#[test]
fn ui() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
}
