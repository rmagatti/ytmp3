// Test that macro fails with invalid input

use ctenv::ctenv;

fn main() {
    // Should fail - no arguments
    let _var = ctenv!();
}