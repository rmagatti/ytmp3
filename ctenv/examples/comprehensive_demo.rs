// Comprehensive example demonstrating ctenv macro features

use ctenv::ctenv;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ctenv Macro Demonstration ===\n");

    // Example 1: Common environment variables
    println!("1. Common Environment Variables:");

    // These might be resolved at compile time if set during build
    let home = ctenv!("HOME");
    println!("   HOME = {}", home);

    let path = ctenv!("PATH");
    println!("   PATH = {} (truncated)", &path[..50.min(path.len())]);

    // Example 2: Application configuration
    println!("\n2. Application Configuration:");

    // Set some example environment variables for demonstration
    std::env::set_var("APP_ENV", "development");
    std::env::set_var("DATABASE_URL", "postgresql://localhost:5432/myapp");
    std::env::set_var("API_KEY", "secret-api-key-12345");
    std::env::set_var("LOG_LEVEL", "info");

    let app_env = ctenv!("APP_ENV");
    let database_url = ctenv!("DATABASE_URL");
    let api_key = ctenv!("API_KEY");
    let log_level = ctenv!("LOG_LEVEL");

    println!("   APP_ENV = {}", app_env);
    println!("   DATABASE_URL = {}", database_url);
    println!("   API_KEY = {}***", &api_key[..3]); // Only show first 3 chars for security
    println!("   LOG_LEVEL = {}", log_level);

    // Example 3: Configuration struct
    println!("\n3. Configuration Struct:");

    #[allow(dead_code)]
    #[derive(Debug)]
    struct AppConfig {
        environment: String,
        database_url: String,
        api_key: String,
        log_level: String,
        port: u16,
    }

    std::env::set_var("PORT", "8080");

    let config = AppConfig {
        environment: ctenv!("APP_ENV"),
        database_url: ctenv!("DATABASE_URL"),
        api_key: ctenv!("API_KEY"),
        log_level: ctenv!("LOG_LEVEL"),
        port: ctenv!("PORT").parse().unwrap_or(3000),
    };

    println!("   Config: {:#?}", config);

    // Example 4: Conditional compilation based on environment
    println!("\n4. Conditional Behavior:");

    match ctenv!("APP_ENV") {
        ref env if env == "development" => {
            println!("   Running in development mode - enabling debug features");
        }
        ref env if env == "production" => {
            println!("   Running in production mode - optimized settings");
        }
        ref env if env == "test" => {
            println!("   Running in test mode - using test database");
        }
        ref env => {
            println!("   Running in {} mode", env);
        }
    }

    // Example 5: Error handling demonstration
    println!("\n5. Error Handling:");

    // This will panic if the variable is not set
    // Uncomment to test: let missing = ctenv!("DEFINITELY_NOT_SET");

    println!("   The macro will panic if a required environment variable is not set");
    println!("   This ensures your application fails fast if misconfigured");

    // Clean up demo environment variables
    std::env::remove_var("APP_ENV");
    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("API_KEY");
    std::env::remove_var("LOG_LEVEL");
    std::env::remove_var("PORT");

    println!("\n=== Demo Complete ===");

    Ok(())
}
