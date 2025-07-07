// Test that basic macro usage compiles correctly

use ctenv::ctenv;

fn main() {
    // Should compile successfully - using PATH which is always available
    let _path = ctenv!("PATH");
    
    // Should work with HOME which is typically available
    let _home = ctenv!("HOME");
    
    println!("All tests passed!");
}