# ctenv

A Rust procedural macro for compile-time environment variable resolution with runtime fallback.

## Features

- **Compile-time resolution**: Environment variables available during compilation are resolved at compile time
- **Runtime fallback**: Variables not available at compile time fall back to runtime resolution
- **.env file support**: Automatically reads from `.env` files during compilation
- **Zero runtime overhead**: When variables are resolved at compile time, there's no runtime cost
- **Panic on missing variables**: Ensures your application fails fast if required environment variables are not set

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
ctenv = "0.1.0"
```

### Basic Usage

```rust
use ctenv::ctenv;

fn main() {
    // This will be resolved at compile time if SUPABASE_URL is available,
    // otherwise it will be resolved at runtime
    let supabase_url = ctenv!("SUPABASE_URL");
    println!("Supabase URL: {}", supabase_url);
    
    // Works with any environment variable
    let database_url = ctenv!("DATABASE_URL");
    let api_key = ctenv!("API_KEY");
    let log_level = ctenv!("LOG_LEVEL");
}
```

### Configuration Struct

```rust
use ctenv::ctenv;

#[derive(Debug)]
struct Config {
    database_url: String,
    api_key: String,
    port: u16,
}

impl Config {
    fn from_env() -> Self {
        Self {
            database_url: ctenv!("DATABASE_URL"),
            api_key: ctenv!("API_KEY"),
            port: ctenv!("PORT").parse().expect("PORT must be a valid number"),
        }
    }
}

fn main() {
    let config = Config::from_env();
    println!("Config: {:#?}", config);
}
```

## How It Works

The `ctenv!` macro follows this resolution order:

1. **Compile-time environment**: First checks if the variable is set in the environment when `cargo build` runs
2. **.env file**: If not found in the environment, checks for a `.env` file in the current directory
3. **Runtime fallback**: If neither of the above work, generates code that calls `std::env::var()` at runtime

### Compile-time Resolution Examples

```bash
# This will resolve SUPABASE_URL at compile time
SUPABASE_URL=https://example.supabase.co cargo build

# Multiple variables
DATABASE_URL=postgres://localhost API_KEY=secret123 cargo build
```

### .env File Support

Create a `.env` file in your project root:

```env
DATABASE_URL=postgresql://localhost:5432/myapp
API_KEY=your-secret-api-key
LOG_LEVEL=info
PORT=8080
```

The macro will automatically read from this file during compilation.

## Error Handling

The macro will panic at runtime if a required environment variable is not set:

```rust
use ctenv::ctenv;

fn main() {
    // This will panic if REQUIRED_VAR is not set
    let value = ctenv!("REQUIRED_VAR");
}
```

This fail-fast behavior ensures your application doesn't start with missing configuration.

## Testing

Run the test suite:

```bash
cd ctenv
./run_tests.sh
```

Or run specific test categories:

```bash
# Unit tests
cargo test --lib

# Integration tests  
cargo test --test integration_tests

# UI tests (compile-time behavior)
cargo test --test ui_tests

# Documentation tests
cargo test --doc
```

## Examples

See the `examples/` directory for comprehensive usage examples:

```bash
# Basic example
cargo run --example ctenv_test

# Comprehensive demonstration
cargo run --example comprehensive_demo
```

## Performance

- **Compile-time resolved variables**: Zero runtime cost
- **Runtime resolved variables**: Single `std::env::var()` call
- **No heap allocations**: Values are embedded as string literals when resolved at compile time

## Comparison with Alternatives

| Feature | ctenv | std::env::var | dotenvy | config crates |
|---------|-------|---------------|---------|---------------|
| Compile-time resolution | ✅ | ❌ | ❌ | ❌ |
| Runtime fallback | ✅ | ✅ | ✅ | ✅ |
| .env file support | ✅ | ❌ | ✅ | ✅ |
| Zero runtime overhead* | ✅ | ❌ | ❌ | ❌ |
| Fail-fast behavior | ✅ | ✅ | ✅ | Varies |

*When variables are resolved at compile time

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.