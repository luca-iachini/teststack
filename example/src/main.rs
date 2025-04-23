use sqlx::{MySqlPool, PgPool};
use stack::{mysql, postgres};

#[postgres]
#[sqlx::test]
async fn invoke(pool: PgPool) {
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("failed to execute query");
}

#[postgres]
#[sqlx::test]
async fn invoke1(pool: PgPool) {
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("failed to execute query");
}

#[postgres]
#[sqlx::test]
async fn invoke3(pool: PgPool) {
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("failed to execute query");
}

#[mysql]
#[sqlx::test]
async fn invoke4(pool: MySqlPool) {
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("failed to execute query");
}

fn main() {
    println!("Hello, world!");
}
