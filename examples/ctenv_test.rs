use ctenv::ctenv;

fn main() {
    // Test the macro with various scenarios

    // This will try to resolve RUST_LOG at compile time if available,
    // otherwise it will be resolved at runtime
    let rust_log = ctenv!("RUST_LOG");
    println!("RUST_LOG = {}", rust_log);

    // Example with a custom environment variable
    let custom_var = ctenv!("MY_CUSTOM_VAR");
    println!("MY_CUSTOM_VAR = {}", custom_var);
}
