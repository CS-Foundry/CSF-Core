use chrono::Utc;
use entity::{entities::logs, Logs};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};
use tokio::time::{interval, Duration};

use crate::routes::settings::load_retention_days;
use crate::AppState;

const PRUNE_INTERVAL: Duration = Duration::from_secs(3600);

pub fn spawn_log_prune_job(state: AppState) {
    tokio::spawn(async move {
        let mut ticker = interval(PRUNE_INTERVAL);
        loop {
            ticker.tick().await;
            prune_once(&state).await;
        }
    });
}

async fn prune_once(state: &AppState) {
    let retention_days = match load_retention_days(state).await {
        Ok(days) => days,
        Err(_) => {
            tracing::warn!("skipping log prune, retention setting unavailable");
            return;
        }
    };

    let cutoff = Utc::now() - chrono::Duration::days(retention_days);

    match delete_logs_older_than(&state.db_conn, cutoff).await {
        Ok(rows_deleted) => {
            tracing::info!(
                rows_deleted = rows_deleted,
                retention_days = retention_days,
                "pruned expired logs"
            );
        }
        Err(error) => {
            tracing::error!(error = %error, "failed to prune expired logs");
        }
    }
}

async fn delete_logs_older_than(
    db: &DbConn,
    cutoff: chrono::DateTime<Utc>,
) -> Result<u64, sea_orm::DbErr> {
    let result = Logs::delete_many()
        .filter(logs::Column::CreatedAt.lt(cutoff))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}
