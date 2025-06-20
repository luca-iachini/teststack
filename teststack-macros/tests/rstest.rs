use teststack::{stack, DbContainer};

#[stack(postgres(random_db_name))]
#[rstest::rstest]
#[tokio::test]
async fn test(container: DbContainer) {
    let pool = sqlx::PgPool::connect(container.conf.url.as_str())
        .await
        .expect("failed to connect to database");
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("failed to execute query");
}

#[stack(postgres(random_db_name))]
#[rstest::rstest]
#[tokio::test]
async fn test_pgpool(pool: sqlx::PgPool) {
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("failed to execute query");
}
