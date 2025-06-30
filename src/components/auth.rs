#![cfg(feature = "hydrate")]

use leptos::{prelude::*, server::codee::string::JsonSerdeCodec};
use leptos_use::storage::use_local_storage;

use ctenv::ctenv;
use supabase_js_rs::{create_client, Credentials};

use serde_wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::domain::entities::auth::{Auth, AuthSession};


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
    let supabase_url = ctenv!("SUPABASE_URL");
    let supabase_anon_key = ctenv!("SUPABASE_ANON_KEY");

    create_client(
        supabase_url,
        supabase_anon_key,
    )
}

/// Signs in a user with email and password.
///
/// # Errors
///
/// Returns an error if authentication fails due to invalid credentials,
/// network issues, or other authentication-related problems.
pub async fn sign_in_with_email(email: String, password: String) -> eyre::Result<Auth> {
    use leptos::logging;

    let client = create_supabase_client();
    let result = client
        .auth()
        .sign_in_with_password(Credentials { email, password })
        .await;

    match result {
        Ok(response) => {
            let auth = Auth::try_from(response.clone())
                .map_err(|e| eyre::eyre!("Failed to parse auth response: {:?}", e))?;
            logging::log!("Sign-in successful, response: {:?}", auth);
            let (_, set_auth_session, _) = use_auth_session();

            if let Some(session) = Option::<AuthSession>::from(&auth) {
                set_auth_session(session);
                logging::log!("Successfully extracted session, storing in local storage");
            } else {
                logging::warn!("No session found in response. User may need to confirm email.");
            }
            Ok(auth)
        }
        Err(err) => {
            logging::error!("Sign-in failed: {:?}", err);
            eyre::bail!("Sign-in failed")
        }
    }
}

/// Signs up a new user with email and password.
///
/// # Errors
///
/// Returns an error if user creation fails due to invalid input,
/// existing user, network issues, or other registration-related problems.
pub async fn sign_up_with_email(email: String, password: String) -> eyre::Result<Auth> {
    use leptos::logging;

    let client = create_supabase_client();
    let result = client.auth().sign_up(Credentials { email, password }).await;

    match result {
        Ok(response) => {
            let auth = Auth::try_from(response.clone())
                .map_err(|e| eyre::eyre!("Failed to parse auth response: {:?}", e))?;
            logging::log!("Sign-up successful, response: {:?}", auth);
            let (_, set_auth_session, _) = use_auth_session();

            if let Some(session) = Option::<AuthSession>::from(&auth) {
                set_auth_session(session);
                logging::log!("Successfully extracted session, storing in local storage");
            } else {
                logging::warn!("No session found in response. User may need to confirm email.");
            }
            Ok(auth)
        }
        Err(err) => {
            logging::error!("Sign-up failed: {:?}", err);
            eyre::bail!("Sign-up failed")
        }
    }
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

impl TryFrom<JsValue> for Auth {
    type Error = JsValue;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        serde_wasm_bindgen::from_value(value).map_err(|e| e.into())
    }
}

