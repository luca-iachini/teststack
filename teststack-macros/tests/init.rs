use std::ops::Deref;

use testcontainers_modules::rabbitmq::RabbitMq;
use teststack::DbContainer;
use teststack::{stack, ContainerPort, CustomContainer};

#[stack(postgres(random_db_name), container(RabbitMq::default()))]
#[tokio::test]
async fn test(conf: TestConfig, rabbit: RabbitConnection) {
    let pool = sqlx::PgPool::connect(conf.url.as_str())
        .await
        .expect("failed to connect to database");
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("failed to execute query");
    rabbit
        .create_channel()
        .await
        .expect("failed to create channel");
}

struct TestConfig {
    url: String,
}

impl teststack::Init<TestConfig> for DbContainer {
    async fn init(self) -> TestConfig {
        TestConfig { url: self.conf.url }
    }
}

struct RabbitConnection(lapin::Connection);

impl Deref for RabbitConnection {
    type Target = lapin::Connection;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl teststack::Init<RabbitConnection> for CustomContainer {
    async fn init(self) -> RabbitConnection {
        let port = self
            .get_host_port_ipv4(ContainerPort::Tcp(5672))
            .await
            .unwrap();

        let url = format!("amqp://guest:guest@localhost:{port}");
        let conn = lapin::Connection::connect(&url, lapin::ConnectionProperties::default())
            .await
            .unwrap();
        RabbitConnection(conn)
    }
}
