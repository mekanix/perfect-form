use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const ACCESS_TOKEN_TTL_SECONDS: u64 = 15 * 60;
const REFRESH_TOKEN_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i32,
    exp: usize,
    iat: usize,
    token_type: String,
}

#[derive(Debug)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}

/// Hash a plain-text password for storage.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

/// Verify a plain-text password against a stored Argon2 hash.
pub fn verify_password(
    password: &str,
    password_hash: &str,
) -> Result<(), argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
}

/// Generate a short-lived access token and a longer-lived refresh token.
pub fn generate_tokens(user_id: i32, secret: &str) -> Result<Tokens, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after the Unix epoch")
        .as_secs() as usize;

    let access_claims = Claims {
        sub: user_id,
        exp: now + ACCESS_TOKEN_TTL_SECONDS as usize,
        iat: now,
        token_type: "access".to_owned(),
    };

    let refresh_claims = Claims {
        sub: user_id,
        exp: now + REFRESH_TOKEN_TTL_SECONDS as usize,
        iat: now,
        token_type: "refresh".to_owned(),
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(Tokens {
        access_token,
        refresh_token,
    })
}

/// Decode an access token and return the user ID it was issued for.
pub fn decode_access_token(token: &str, secret: &str) -> Result<i32, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    if token_data.claims.token_type != "access" {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
    }

    Ok(token_data.claims.sub)
}

use sea_orm::{DatabaseConnection, EntityTrait, ModelTrait};
use topcoat::{
    Result,
    context::Cx,
    cookie::{Cookies, cookies},
    router::error::SeeOther,
    router::response::{IntoResponse, Response},
    view::View,
};

use crate::{config::Config, entity::user};

/// Re-exported proc-macro attribute for guarding routes by user roles.
pub use perfect_form_macros::permissions;

/// Either a view to render or a redirect for an auth-related page.
pub enum Page {
    View(View),
    Redirect(SeeOther),
}

impl IntoResponse for Page {
    fn into_response(self, cx: &Cx) -> Result<Response> {
        match self {
            Page::View(view) => view.into_response(cx),
            Page::Redirect(redirect) => redirect.into_response(cx),
        }
    }
}

/// Look up the currently logged-in user from the access-token cookie.
pub async fn current_user(
    cx: &Cx,
    db: &DatabaseConnection,
    cfg: &Config,
) -> Result<Option<user::Model>> {
    let token = match cookies(cx).get("access_token") {
        Some(cookie) => cookie.value().to_owned(),
        None => return Ok(None),
    };

    let user_id = match decode_access_token(&token, &cfg.jwt_secret) {
        Ok(id) => id,
        Err(_) => return Ok(None),
    };

    Ok(user::Entity::find_by_id(user_id).one(db).await?)
}

/// A role-based access requirement.
///
/// `Any` is satisfied when the user holds at least one of the listed roles.
/// `All` is satisfied when the user holds every listed role.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum RoleRequirement {
    Any(&'static [&'static str]),
    All(&'static [&'static str]),
}

/// The result of an authorization check.
#[allow(dead_code)]
pub enum AuthOutcome {
    /// The request is allowed and this is the authenticated user.
    User(user::Model),
    /// The user is not logged in.
    LoginRedirect,
    /// The user is logged in but does not meet the role requirements.
    ForbiddenRedirect,
}

/// Enforce role requirements for the current request.
///
/// Returns the authenticated user on success. If the user is not logged in,
/// returns `AuthOutcome::LoginRedirect`. If the user is logged in but fails any
/// requirement, returns `AuthOutcome::ForbiddenRedirect`. Admin users bypass
/// all role checks.
pub async fn require_roles(
    cx: &Cx,
    db: &DatabaseConnection,
    cfg: &Config,
    requirements: &[RoleRequirement],
) -> Result<AuthOutcome> {
    let Some(user) = current_user(cx, db, cfg).await? else {
        return Ok(AuthOutcome::LoginRedirect);
    };

    if user.admin {
        return Ok(AuthOutcome::User(user));
    }

    let roles: Vec<crate::entity::role::Model> = user
        .find_related(crate::entity::role::Entity)
        .all(db)
        .await?;
    let role_names: Vec<&str> = roles.iter().map(|r| r.name.as_str()).collect();

    for requirement in requirements {
        let satisfied = match requirement {
            RoleRequirement::Any(roles) => roles.iter().any(|role| role_names.contains(role)),
            RoleRequirement::All(roles) => roles.iter().all(|role| role_names.contains(role)),
        };

        if !satisfied {
            return Ok(AuthOutcome::ForbiddenRedirect);
        }
    }

    Ok(AuthOutcome::User(user))
}
