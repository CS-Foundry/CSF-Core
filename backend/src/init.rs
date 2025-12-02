use entity::{key, user, Key, User};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::auth::crypto::{generate_salt, hash_password, RsaKeyPair};

pub async fn initialize_database(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Initializing database with default data...");

    // 1. Create RSA key pair if not exists
    let key_exists = Key::find()
        .filter(key::Column::Name.eq("main"))
        .one(db)
        .await?
        .is_some();

    if !key_exists {
        tracing::info!("Creating RSA key pair...");
        let key_pair = RsaKeyPair::generate()?;
        let key_id = Uuid::new_v4();
        let new_key = key::ActiveModel {
            id: ActiveValue::Set(key_id),
            name: ActiveValue::Set("main".to_string()),
            private_key: ActiveValue::Set(key_pair.private_key),
        };
        Key::insert(new_key).exec_without_returning(db).await?;
        tracing::info!("RSA key pair created successfully");
    } else {
        tracing::info!("RSA key pair already exists");
    }

    // 2. Create admin user if not exists
    let admin_exists = User::find()
        .filter(user::Column::Name.eq("admin@local.com"))
        .one(db)
        .await?
        .is_some();

    if !admin_exists {
        tracing::info!("Creating default admin user...");
        let salt = generate_salt();
        let hashed_password = hash_password("admin", &salt)?;
        let user_id = Uuid::new_v4();

        let admin_user = user::ActiveModel {
            id: ActiveValue::Set(user_id),
            name: ActiveValue::Set("admin@local.com".to_string()),
            password: ActiveValue::Set(hashed_password),
            salt: ActiveValue::Set(salt),
        };

        User::insert(admin_user).exec_without_returning(db).await?;
        tracing::info!("Default admin user created: admin@local.com / admin");
    } else {
        tracing::info!("Admin user already exists");
    }

    tracing::info!("Database initialization completed");
    Ok(())
}
