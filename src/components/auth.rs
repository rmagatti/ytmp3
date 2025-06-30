#![cfg(feature = "hydrate")]

use leptos::{prelude::*, server::codee::string::JsonSerdeCodec};
use leptos_use::storage::use_local_storage;
use serde::{Deserialize, Serialize};

use supabase_js_rs::{create_client, Credentials};

use serde_wasm_bindgen;
use wasm_bindgen::JsValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Auth {
    pub data: AuthData,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthData {
    pub user: Option<User>,
    pub session: Option<Session>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub aud: String,
    pub role: String,
    pub email: String,
    pub email_confirmed_at: Option<String>,
    pub phone: String,
    pub confirmation_sent_at: Option<String>,
    pub confirmed_at: Option<String>,
    pub last_sign_in_at: Option<String>,
    pub app_metadata: AppMetadata,
    pub user_metadata: UserMetadata,
    pub identities: Vec<Identity>,
    pub created_at: String,
    pub updated_at: String,
    pub is_anonymous: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
    pub expires_at: i64,
    pub refresh_token: String,
    pub user: User,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Identity {
    pub identity_id: String,
    pub id: String,
    pub user_id: String,
    pub identity_data: IdentityData,
    pub provider: String,
    pub last_sign_in_at: String,
    pub created_at: String,
    pub updated_at: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentityData {
    pub email: String,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub sub: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserMetadata {
    pub email: String,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub sub: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppMetadata {
    pub provider: String,
    pub providers: Vec<String>,
}

// Simplified session struct for local storage
#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSession {
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub email: String,
    pub role: String,
    pub expires_at: i64,
    pub email_confirmed_at: Option<String>,
    pub last_sign_in_at: Option<String>,
    pub is_anonymous: bool,
}

impl From<&Auth> for Option<AuthSession> {
    fn from(auth: &Auth) -> Self {
        let session = auth.data.session.as_ref()?;
        let user = &session.user;

        Some(AuthSession {
            user_id: user.id.clone(),
            access_token: session.access_token.clone(),
            refresh_token: session.refresh_token.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            expires_at: session.expires_at,
            email_confirmed_at: user.email_confirmed_at.clone(),
            last_sign_in_at: user.last_sign_in_at.clone(),
            is_anonymous: user.is_anonymous,
        })
    }
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
