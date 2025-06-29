#![cfg(feature = "hydrate")]

use leptos::{prelude::*, server::codee::string::JsonSerdeCodec};
use leptos_use::storage::use_local_storage;
use serde::{Deserialize, Serialize};

use supabase_js_rs::{create_client, Credentials};

use wasm_bindgen::JsValue;

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: String,
}

pub fn use_auth_session() -> (
    Signal<AuthSession>,
    WriteSignal<AuthSession>,
    impl Fn() + Clone,
) {
    let (auth_session, set_auth_session, clear) =
        use_local_storage::<AuthSession, JsonSerdeCodec>("supabase.auth.token");
    (auth_session, set_auth_session, clear)
}

pub fn create_supabase_client() -> supabase_js_rs::SupabaseClient {
    create_client(
        dotenvy_macro::dotenv!("SUPABASE_URL"),
        dotenvy_macro::dotenv!("SUPABASE_ANON_KEY"),
    )
}

/// Signs in a user with email and password.
///
/// # Errors
///
/// Returns an error if authentication fails due to invalid credentials,
/// network issues, or other authentication-related problems.
pub async fn sign_in_with_email(email: String, password: String) -> Result<JsValue, JsValue> {
    let client = create_supabase_client();
    client
        .auth()
        .sign_in_with_password(Credentials { email, password })
        .await
}

/// Signs up a new user with email and password.
///
/// # Errors
///
/// Returns an error if user creation fails due to invalid input,
/// existing user, network issues, or other registration-related problems.
pub async fn sign_up_with_email(email: String, password: String) -> Result<JsValue, JsValue> {
    let client = create_supabase_client();
    client.auth().sign_up(Credentials { email, password }).await
}

/// Signs out the current user.
///
/// # Errors
///
/// Returns an error if the sign-out operation fails due to network issues
/// or other authentication service problems.
pub async fn sign_out() -> Result<JsValue, JsValue> {
    let client = create_supabase_client();
    let result = client.auth().sign_out().await;
    
    // Clear local storage on successful sign out
    if result.is_ok() {
        let (_, _, clear) = use_auth_session();
        clear();
    }
    
    result
}
