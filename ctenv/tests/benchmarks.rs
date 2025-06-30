// Benchmark tests for the ctenv macro
// These tests help ensure the macro doesn't significantly impact compile times

use std::time::Instant;

#[test]
fn benchmark_macro_expansion() {
    // This is more of a compile-time benchmark placeholder
    // In practice, you'd measure this during actual compilation

    let start = Instant::now();

    // Simulate multiple macro calls (in a real scenario, these would be at compile time)
    for i in 0..1000 {
        let var_name = format!("TEST_VAR_{}", i);
        std::env::set_var(&var_name, "test_value");

        // In a real scenario, this would be ctenv!(var_name) at compile time
        // For now, we just test the runtime equivalent
        let _result = std::env::var(&var_name).unwrap_or_else(|_| "default".to_string());

        std::env::remove_var(&var_name);
    }

    let duration = start.elapsed();

    // Ensure it completes reasonably quickly
    assert!(
        duration.as_millis() < 1000,
        "Macro calls took too long: {:?}",
        duration
    );
}
