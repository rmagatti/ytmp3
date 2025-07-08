use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, emit_warning};
use quote::quote;
use syn::{parse_macro_input, LitStr};

/// A procedural macro that resolves environment variables at compile time when possible,
/// with runtime fallback.
///
/// Behavior:
/// 1. First tries to resolve from compile-time environment (e.g., `SUPABASE_URL=... cargo build`)
/// 2. Then tries to resolve from .env file at compile time
/// 3. If neither found, generates code that will call `std::env::var` at runtime
///
/// Usage: `ctenv!("ENVIRONMENT_VARIABLE_NAME")`
///
/// # Examples
///
/// ```rust,no_run
/// use ctenv::ctenv;
///
/// // This will be resolved at compile time if SUPABASE_URL is available,
/// // otherwise it will be resolved at runtime or panic if not set.
/// let supabase_url = ctenv!("SUPABASE_URL");
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn ctenv(input: TokenStream) -> TokenStream {
    // Parse the key passed to the macro
    let var = parse_macro_input!(input as LitStr);
    let var_name = var.value();

    // 1. Real compile-time environment (cargo build SUPABASE_URL=...)
    if let Ok(val) = std::env::var(&var_name) {
        emit_warning!(
            var.span(),
            "found value for {} in environment at compile time",
            var_name
        );
        return quote!(#val.to_string()).into();
    }

    // 2. .env file present at compile time?
    if let Ok(iter) = dotenvy::from_path_iter(".env") {
        for item in iter.flatten() {
            if item.0 == var_name {
                emit_warning!(var.span(), "found value for {} in .env at compile time", var_name);
                let val = item.1;
                return quote!(#val.to_string()).into();
            }
        }
    }

    // 3. Fallback – generate code that tries at run-time
    // (still panics if the var is missing then)
    emit_warning!(
        var.span(),
        "deferring resolution of {} to runtime",
        var_name
    );
    let tokens = quote! {
        ::std::env::var(#var_name)
             .expect(concat!("Environment variable ", #var_name, " is not set"))
    };
    tokens.into()
}
