use sqlx::{MySqlPool, PgPool};
use teststack::stack;

#[stack(postgres)]
#[sqlx::test]
async fn test_postgres(pool: PgPool) {
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("failed to execute query");
}

#[stack(mysql)]
#[sqlx::test]
async fn test_mysql(pool: MySqlPool) {
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("failed to execute query");
}
