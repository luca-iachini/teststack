# Teststack

Teststack is a Rust utility crate that simplifies the setup and management of reusable test containers using the  [testcontainers](https://docs.rs/testcontainers/latest/testcontainers/) library. It starts each container once per test suite and shares it across tests, reducing overhead and complexity.

# Features

- Singleton containers per image type across tests
- Automatic cleanup at the end of the test run (or on Ctrl+C)
- Pluggable container support: PostgreSQL, MySQL, and custom containers
- Easy integration with test frameworks (sqlx::test, rstest)

# Getting Started

Add `teststack` to your Cargo.toml:

```toml
[dev-dependencies]
teststack = { version = "0.1", features = ["postgres"] }
```

# Example: Shared Container Across Tests

In the example below, both tests share the same `Postgres` container instance. This reduces startup overhead and speeds up test execution. The container is automatically shut down at the end of the test harness.

```rust
use sqlx::{MySqlPool, PgPool};
use teststack::stack;

#[stack(postgres(random_db_name))]
#[sqlx::test]
async fn test_postgres(pool: PgPool) {
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("failed to execute query");
}

#[stack(postgres(random_db_name))]
#[sqlx::test]
async fn test_postgres_2(pool: PgPool) {
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("failed to execute query");
}
```

# Custom Containers and Configuration

The following example demonstrates how to run a `RabbitMq` container using teststack. It shows how to customize an existing `testcontainers_module` image with the `ImageExt` trait for advanced configuration.

```rust
use testcontainers_modules::rabbitmq::RabbitMq;
use testcontainers_modules::testcontainers::{ContainerRequest, ImageExt};
use teststack::DbContainer;
use teststack::{ContainerPort, CustomContainer, stack};

#[stack(container(rabbit()))]
#[tokio::test]
async fn test(rabbit: RabbitConnection) {
    rabbit
        .0
        .create_channel()
        .await
        .expect("failed to create channel");
}

fn rabbit() -> ContainerRequest<RabbitMq> {
    RabbitMq::default().with_tag("3.11.0-alpine")
}

struct RabbitConnection(lapin::Connection);

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
```

If you don't need any custom configuration, you can simplify the container setup by returning the image directly:

```rust
use testcontainers_modules::rabbitmq::RabbitMq;

fn rabbit() -> RabbitMq {
    RabbitMq::default()
}

```

# Database Utilities

Enable a specific feature (`postgres`, `mysql`) to run dedicated database containers:

```rust
use teststack::{DbContainer, stack};

#[stack(postgres(random_db_name))]
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
```

# Design Overview

All containers are shared per image type.
Cleanup is performed once, when the test process exits or receives a Ctrl+C signal.


# Cleanup Strategy

Test containers are gracefully cleaned up on process exit using ctor and dtor, with support for both async and blocking cleanup.

# Features

* `postgres` – enable PostgreSQL test container support
* `mysql` – enable MySQL test container support

