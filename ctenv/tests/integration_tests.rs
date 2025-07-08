use ctenv::ctenv;

#[test]
fn test_compile_time_env_var() {
    // This test verifies that environment variables set during compilation
    // are properly resolved at compile time.
    // Note: The actual value will depend on what's set during `cargo test`

    // Set an env var for this test
    std::env::set_var("CTENV_TEST_VAR", "compile_time_value");

    // This should resolve at runtime since we're in a test context
    let result = ctenv!("CTENV_TEST_VAR");
    assert_eq!(result, "compile_time_value");

    // Clean up
    std::env::remove_var("CTENV_TEST_VAR");
}

#[test]
fn test_runtime_fallback() {
    // Test that variables not available at compile time fall back to runtime resolution
    std::env::set_var("CTENV_RUNTIME_VAR", "runtime_value");

    let result = ctenv!("CTENV_RUNTIME_VAR");
    assert_eq!(result, "runtime_value");

    std::env::remove_var("CTENV_RUNTIME_VAR");
}

#[test]
#[should_panic(expected = "Environment variable CTENV_MISSING_VAR is not set")]
fn test_missing_env_var_panics() {
    // Ensure the variable is not set
    std::env::remove_var("CTENV_MISSING_VAR");

    // This should panic at runtime
    let _result = ctenv!("CTENV_MISSING_VAR");
}

#[cfg(test)]
mod dotenv_tests {
    // Note: .env file testing is complex because proc macros run at compile time
    // See dotenv_integration.rs for more comprehensive .env testing
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_env_var() {
        std::env::set_var("CTENV_EMPTY_VAR", "");

        let result = ctenv!("CTENV_EMPTY_VAR");
        assert_eq!(result, "");

        std::env::remove_var("CTENV_EMPTY_VAR");
    }

    #[test]
    fn test_env_var_with_spaces() {
        std::env::set_var("CTENV_SPACES_VAR", "value with spaces");

        let result = ctenv!("CTENV_SPACES_VAR");
        assert_eq!(result, "value with spaces");

        std::env::remove_var("CTENV_SPACES_VAR");
    }

    #[test]
    fn test_env_var_with_special_chars() {
        std::env::set_var("CTENV_SPECIAL_VAR", "value=with:special;chars");

        let result = ctenv!("CTENV_SPECIAL_VAR");
        assert_eq!(result, "value=with:special;chars");

        std::env::remove_var("CTENV_SPECIAL_VAR");
    }
}
