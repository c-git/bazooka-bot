use std::time::Duration;

use crate::{
    db::{load_kv, save_kv},
    model::schedule::UnixTimestamp,
};
use human_time::ToHumanTimeString;
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

pub async fn last_heartbeat_info(db_pool: sqlx::PgPool) -> String {
    match load_kv(&db_pool, KEY).await {
        Some(db_value) => match UnixTimestamp::from_db_fmt(&db_value) {
            Ok(last_heartbeat) => {
                let Ok(now) = UnixTimestamp::now() else {
                    return format!(
                        "Last Heartbeat: {last_heartbeat} but Failed to get current timestamp"
                    );
                };
                let seconds_since_last_heartbeat = now.0 - last_heartbeat.0;
                if seconds_since_last_heartbeat < 0 {
                    return format!(
                        "Last heartbeat in the future?! Last heartbeat: {last_heartbeat}, Now: {now}"
                    );
                }
                let Ok(seconds_since_last_heartbeat) = seconds_since_last_heartbeat.try_into()
                else {
                    // Invalid u64
                    return format!(
                        "Invalid u64!!! Seconds since heartbeat: {seconds_since_last_heartbeat},  Last heartbeat: {last_heartbeat}, Now: {now}"
                    );
                };
                let downtime = Duration::from_secs(seconds_since_last_heartbeat);
                format!(
                    "Downtime: {}\nLast Heartbeat: {last_heartbeat}\nNow: {now}",
                    downtime.to_human_time_string()
                )
            }
            Err(err) => {
                error!(?err);
                "Error Loading Last Heartbeat".to_string()
            }
        },
        None => "First run".to_string(),
    }
}
