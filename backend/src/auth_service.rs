use chrono::{DateTime, Utc};
use entity::{invalid_jwt, key, user, InvalidJwt, Key, User};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use thiserror::Error;
use uuid::Uuid;

use crate::auth::{
    crypto::{
        decrypt_password, generate_salt, hash_password, verify_password, CryptoError, RsaKeyPair,
    },
    jwt::create_jwt,
};

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error("Crypto error: {0}")]
    CryptoError(#[from] CryptoError),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User already exists")]
    UserAlreadyExists,
}

pub type AuthResult<T> = Result<T, AuthError>;

#[derive(Clone)]
pub struct AuthService {
    db: DatabaseConnection,
}

impl AuthService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn register_user(
        &self,
        username: String,
        encrypted_password: String,
    ) -> AuthResult<String> {
        // Check if user already exists
        let existing_user = User::find()
            .filter(user::Column::Name.eq(&username))
            .one(&self.db)
            .await?;

        if existing_user.is_some() {
            return Err(AuthError::UserAlreadyExists);
        }

        // Get RSA private key for password decryption
        let rsa_key = self.get_or_create_rsa_key("main").await?;

        // Decrypt the password
        let password = decrypt_password(&encrypted_password, &rsa_key.private_key)?;

        // Generate salt and hash password
        let salt = generate_salt();
        let hashed_password = hash_password(&password, &salt)?;

        // Create user with pre-generated UUID (SQLite can't auto-generate UUIDs)
        let user_id = Uuid::new_v4();
        let new_user = user::ActiveModel {
            id: ActiveValue::Set(user_id),
            name: ActiveValue::Set(username.clone()),
            password: ActiveValue::Set(hashed_password),
            salt: ActiveValue::Set(salt),
        };

        // Insert without retrieving last_insert_id (which doesn't work with UUID PKs in SQLite)
        User::insert(new_user)
            .exec_without_returning(&self.db)
            .await?;

        // Create JWT token using our pre-generated user ID
        let token = create_jwt(user_id, username)?;
        Ok(token)
    }

    pub async fn login_user(
        &self,
        username: String,
        encrypted_password: String,
    ) -> AuthResult<String> {
        // Find user
        let user = User::find()
            .filter(user::Column::Name.eq(&username))
            .one(&self.db)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        // Get RSA private key for password decryption
        let rsa_key = self.get_or_create_rsa_key("main").await?;

        // Decrypt the password
        let password = decrypt_password(&encrypted_password, &rsa_key.private_key)?;

        // Verify password
        if !verify_password(&password, &user.salt, &user.password)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Create JWT token
        let token = create_jwt(user.id, user.name)?;
        Ok(token)
    }

    pub async fn logout_user(&self, token: String, exp: DateTime<Utc>) -> AuthResult<()> {
        // Add token to blacklist with pre-generated UUID
        let jwt_id = Uuid::new_v4();
        let invalid_jwt = invalid_jwt::ActiveModel {
            id: ActiveValue::Set(jwt_id),
            token: ActiveValue::Set(token),
            exp: ActiveValue::Set(exp.naive_utc()),
        };

        // Insert without retrieving last_insert_id (which doesn't work with UUID PKs in SQLite)
        InvalidJwt::insert(invalid_jwt)
            .exec_without_returning(&self.db)
            .await?;
        Ok(())
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> AuthResult<user::Model> {
        User::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or(AuthError::UserNotFound)
    }

    pub async fn get_or_create_rsa_key(&self, key_name: &str) -> AuthResult<RsaKeyPair> {
        // Try to find existing key
        if let Some(existing_key) = Key::find()
            .filter(key::Column::Name.eq(key_name))
            .one(&self.db)
            .await?
        {
            return Ok(RsaKeyPair::from_private_key(&existing_key.private_key)?);
        }

        // Generate new key pair
        let key_pair = RsaKeyPair::generate()?;

        // Save to database with pre-generated UUID
        let key_id = Uuid::new_v4();
        let new_key = key::ActiveModel {
            id: ActiveValue::Set(key_id),
            name: ActiveValue::Set(key_name.to_string()),
            private_key: ActiveValue::Set(key_pair.private_key.clone()),
        };

        // Insert without retrieving last_insert_id (which doesn't work with UUID PKs in SQLite)
        Key::insert(new_key)
            .exec_without_returning(&self.db)
            .await?;

        Ok(key_pair)
    }

    pub async fn get_public_key(&self, key_name: &str) -> AuthResult<String> {
        let key_pair = self.get_or_create_rsa_key(key_name).await?;
        Ok(key_pair.public_key)
    }

    #[allow(dead_code)]
    pub async fn cleanup_expired_tokens(&self) -> AuthResult<()> {
        let now = Utc::now().naive_utc();
        InvalidJwt::delete_many()
            .filter(invalid_jwt::Column::Exp.lt(now))
            .exec(&self.db)
            .await?;

        Ok(())
    }
}
