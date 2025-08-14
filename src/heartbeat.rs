use crate::{db::save_kv, model::schedule::UnixTimestamp};
use tracing::{error, info};

const KEY: &str = "HEARTBEAT";

pub fn start_heartbeat(db_pool: sqlx::PgPool) {
    shuttle_runtime::tokio::spawn(async move {
        info!("Heartbeat started");
        loop {
            let timestamp = match UnixTimestamp::now() {
                Ok(x) => x,
                Err(err) => {
                    error!(?err, "failed to get timestamp heartbeat stopping");
                    break;
                }
            };
            save_kv(&db_pool, KEY, timestamp.to_db_fmt()).await;
            shuttle_runtime::tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
}

pub fn time_since_last_heartbeat(db_pool: sqlx::PgPool) -> String {
    // TODO 1: Get last heartbeat from DB
    "First run".to_string()
}
