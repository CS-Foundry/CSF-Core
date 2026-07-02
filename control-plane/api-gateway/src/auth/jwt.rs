use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub exp: i64,    // Expiration time
    pub iat: i64,    // Issued at
    pub user_id: Uuid,
    pub username: String,
}

impl Claims {
    pub fn new(user_id: Uuid, username: String) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(24); // Token expires in 24 hours

        Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            user_id,
            username,
        }
    }
}

pub fn create_jwt(user_id: Uuid, username: String) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(user_id, username);
    let secret = get_jwt_secret();
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn verify_jwt(token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let secret = get_jwt_secret();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
}

fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecTicketClaims {
    pub workload_id: Uuid,
    pub user_id: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub purpose: String,
}

impl ExecTicketClaims {
    pub fn new(workload_id: Uuid, user_id: Uuid) -> Self {
        let now = Utc::now();
        ExecTicketClaims {
            workload_id,
            user_id,
            exp: (now + Duration::seconds(30)).timestamp(),
            iat: now.timestamp(),
            purpose: "workload-exec".to_string(),
        }
    }
}

pub fn create_exec_ticket(
    workload_id: Uuid,
    user_id: Uuid,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = ExecTicketClaims::new(workload_id, user_id);
    let secret = get_jwt_secret();
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn verify_exec_ticket(
    token: &str,
    workload_id: Uuid,
) -> Result<ExecTicketClaims, jsonwebtoken::errors::Error> {
    let secret = get_jwt_secret();
    let data = decode::<ExecTicketClaims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    if data.claims.purpose != "workload-exec" || data.claims.workload_id != workload_id {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
    }

    Ok(data.claims)
}
