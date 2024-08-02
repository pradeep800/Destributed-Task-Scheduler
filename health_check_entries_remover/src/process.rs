use sqlx::PgPool;

pub async fn process(pool: &PgPool) {
    sqlx::query!(
        "
        DELETE FROM health_check_entries 
        WHERE worker_finished= true
        AND last_time_health_check <= NOW()- INTERVAl '20 MINUTES'
        "
    )
    .execute(pool)
    .await
    .unwrap();
}
