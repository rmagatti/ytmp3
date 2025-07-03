use leptos::{prelude::*, server::codee::string::JsonSerdeCodec};
use leptos_use::{use_cookie_with_options, UseCookieOptions};
use leptos::logging;

#[cfg(feature = "hydrate")]
use ctenv::ctenv;
#[cfg(feature = "hydrate")]
use supabase_js_rs::{create_client, Credentials};

#[cfg(feature = "hydrate")]
use serde_wasm_bindgen;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsValue;

use crate::domain::entities::auth::AuthSession;
#[cfg(feature = "hydrate")]
use crate::domain::entities::auth::Auth;
#[cfg(feature = "hydrate")]
use crate::domain::entities::auth::User;
#[cfg(feature = "hydrate")]
use serde::Deserialize;

#[cfg(feature = "hydrate")]
#[derive(Deserialize)]
struct GetUserResponse {
    data: UserData,
}

#[cfg(feature = "hydrate")]
#[derive(Deserialize)]
struct UserData {
    user: User,
}

pub fn use_auth_session() -> (Signal<Option<AuthSession>>, WriteSignal<Option<AuthSession>>) {
    let (auth_session, set_auth_session) = use_cookie_with_options::<AuthSession, JsonSerdeCodec>(
        "supabase.auth.token",
        UseCookieOptions::default()
            .max_age(60 * 60 * 24 * 7) // 7 days
            .path("/"),
    );

    logging::log!("use_auth_session called, current session: {:?}", auth_session);
    (auth_session, set_auth_session)
}

#[cfg(feature = "hydrate")]
pub fn create_supabase_client() -> supabase_js_rs::SupabaseClient {
    let supabase_url = ctenv!("SUPABASE_URL");
    let supabase_anon_key = ctenv!("SUPABASE_ANON_KEY");

    create_client(&supabase_url, &supabase_anon_key)
}

/// Signs in a user with email and password.
///
/// # Errors
///
/// Returns an error if authentication fails due to invalid credentials,
/// network issues, or other authentication-related problems.
#[cfg(feature = "hydrate")]
pub async fn sign_in_with_email(
    email: String,
    password: String,
    set_auth_session: WriteSignal<Option<AuthSession>>,
) -> eyre::Result<Auth> {
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

            if let Some(session) = Option::<AuthSession>::from(&auth) {
                set_auth_session.set(Some(session));
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
#[cfg(feature = "hydrate")]
pub async fn sign_up_with_email(
    email: String,
    password: String,
    set_auth_session: WriteSignal<Option<AuthSession>>,
) -> eyre::Result<Auth> {
    let client = create_supabase_client();
    let result = client.auth().sign_up(Credentials { email, password }).await;

    match result {
        Ok(response) => {
            let auth = Auth::try_from(response.clone())
                .map_err(|e| eyre::eyre!("Failed to parse auth response: {:?}", e))?;
            logging::log!("Sign-up successful, response: {:?}", auth);

            if let Some(session) = Option::<AuthSession>::from(&auth) {
                set_auth_session.set(Some(session));
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
#[cfg(feature = "hydrate")]
pub async fn sign_out(
    set_auth_session: WriteSignal<Option<AuthSession>>,
) -> Result<JsValue, JsValue> {
    let client = create_supabase_client();
    let result = client.auth().sign_out().await;

    logging::log!("Sign-out result: {:?}", result);

    set_auth_session.set(None);

    result
}

#[cfg(feature = "hydrate")]
pub async fn get_user(token: &str) -> Result<User, JsValue> {
    let client = create_supabase_client();
    let result = client.auth().get_user(Some(token)).await?;
    let response: GetUserResponse = serde_wasm_bindgen::from_value(result).map_err(JsValue::from)?;
    Ok(response.data.user)
}

#[cfg(feature = "hydrate")]
impl TryFrom<JsValue> for Auth {
    type Error = JsValue;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        serde_wasm_bindgen::from_value(value).map_err(|e| e.into())
    }
}