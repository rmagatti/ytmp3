// Test that macro fails with invalid argument type

use ctenv::ctenv;

fn main() {
    // Should fail - not a string literal
    let var_name = "SOME_VAR";
    let _var = ctenv!(var_name);
}