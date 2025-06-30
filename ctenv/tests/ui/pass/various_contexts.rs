// Test that the macro works in various contexts

use ctenv::ctenv;

fn main() {
    // In variable assignment
    let _home = ctenv!("HOME");
    
    // In function call
    println!("Path length: {}", ctenv!("PATH").len());
    
    // In struct initialization
    struct Config {
        home_dir: String,
    }
    
    let _config = Config {
        home_dir: ctenv!("HOME"),
    };
    
    // In match expression
    match ctenv!("HOME") {
        env => println!("Environment: {}", env),
    }
    
    println!("Context tests passed!");
}