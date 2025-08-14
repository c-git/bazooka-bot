use tracing::info;

pub fn start_heartbeat(db_pool: sqlx::PgPool) {
    shuttle_runtime::tokio::spawn(async move {
        info!("Heartbeat started");
        loop {
            // TODO 1: Save time of last heartbeat to DB
            println!("beat");
            shuttle_runtime::tokio::time::sleep(std::time::Duration::from_secs(6)).await;
        }
    });
}

pub fn time_since_last_heartbeat(db_pool: sqlx::PgPool) -> String {
    // TODO 1: Get last heartbeat from DB
    "First run".to_string()
}
