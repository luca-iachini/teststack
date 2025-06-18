use sqlx::PgPool;
use teststack::stack;

#[stack(postgres(db_name = "test"))]
#[tokio::test]
async fn test(pool: PgPool) {
    let db_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(db_name, "test");
}
