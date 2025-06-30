// Tests for compile-time behavior of the ctenv macro
// These tests verify that the macro compiles correctly and handles various scenarios

#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();

    // Test that valid macro usage compiles
    t.pass("tests/ui/pass/*.rs");

    // Test that invalid macro usage fails to compile with proper error messages
    t.compile_fail("tests/ui/fail/*.rs");
}
