use tracing::{debug, error, info};

pub async fn save_kv(pool: &sqlx::PgPool, key: &str, value: String) {
    let query = sqlx::query!(
        "\
INSERT INTO kv_store (id, content)
    VALUES ($1, $2)
    ON CONFLICT(id)
    DO UPDATE SET
    content = EXCLUDED.content;",
        key,
        value
    );
    match query.execute(pool).await {
        Ok(query_result) => {
            if query_result.rows_affected() == 1 {
                debug!("Save completed for key: {key}");
            } else {
                error!(
                    ?key,
                    "Expected 1 row to be affected by save but got: {}",
                    query_result.rows_affected()
                )
            }
        }
        Err(err_msg) => error!(
            ?err_msg,
            "Failed to save content for key: {key} to kv store"
        ),
    }
}

pub async fn load_kv(pool: &sqlx::PgPool, key: &str) -> Option<String> {
    match sqlx::query!("SELECT content FROM kv_store where id = $1", key)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(record)) => Some(record.content),
        Ok(None) => {
            info!("No content found in DB for key: {key}");
            None
        }
        Err(err_msg) => {
            error!(?err_msg, "Failed to get content for key: {key}");
            None
        }
    }
}
