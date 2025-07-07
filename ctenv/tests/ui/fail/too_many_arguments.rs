// Test that macro fails with too many arguments

use ctenv::ctenv;

fn main() {
    // Should fail - too many arguments
    let _var = ctenv!("VAR1", "VAR2");
}