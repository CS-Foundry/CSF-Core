use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DbConn, DbErr};
use std::env;
use std::path::Path;

pub async fn establish_connection() -> Result<DbConn, DbErr> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create SQLite database file if it doesn't exist
    if database_url.starts_with("sqlite:") {
        let db_path = database_url.strip_prefix("sqlite:").unwrap();
        if !Path::new(db_path).exists() {
            tracing::info!("Creating SQLite database file: {}", db_path);
            // Create parent directories if they don't exist
            if let Some(parent) = Path::new(db_path).parent() {
                std::fs::create_dir_all(parent).expect("Failed to create database directory");
            }
            // Create empty database file
            std::fs::File::create(db_path).expect("Failed to create database file");
        }
    }

    tracing::info!("Connecting to database: {}", database_url);
    let db_conn = Database::connect(&database_url).await?;

    // Run pending migrations
    tracing::info!("Running database migrations...");
    Migrator::up(&db_conn, None).await?;
    tracing::info!("Database migrations completed successfully");

    Ok(db_conn)
}
