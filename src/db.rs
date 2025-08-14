use tracing::{error, info};

pub async fn save_kv(pool: &sqlx::PgPool, key: String, value: String) {
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
                info!("Save completed for key: {key}");
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
