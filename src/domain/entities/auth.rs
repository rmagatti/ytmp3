use serde::{Deserialize, Serialize};

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
#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
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

impl From<Auth> for Option<AuthSession> {
    fn from(auth: Auth) -> Self {
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
