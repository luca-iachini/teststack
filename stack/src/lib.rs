use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use testcontainers::{Container, Image};

pub type GenericContainer = Arc<dyn RunningContainer>;

lazy_static::lazy_static! {
    static ref CONTAINERS: Mutex<HashMap<TypeId, GenericContainer>> = Mutex::new(HashMap::new());
}

// Remove all containers when the test harness exits
#[ctor::dtor]
fn remove_containers() {
    let containers = CONTAINERS.try_lock().unwrap();
    for (_, container) in containers.iter() {
        let container_id = container.id();
        let status = std::process::Command::new("docker")
            .arg("rm")
            .arg("-f")
            .arg(container_id)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("Failed to remove a container");
        assert!(status.success(), "Failed to remove container");
    }
}

pub trait RunningContainer: Send + Sync {
    fn id(&self) -> &str;
}

impl<T: Image> RunningContainer for Container<T> {
    fn id(&self) -> &str {
        self.id()
    }
}

#[cfg(feature = "postgres")]
mod postgres {
    use std::any::TypeId;
    use std::sync::{Arc, OnceLock};

    use testcontainers::{Container, runners::SyncRunner};
    use testcontainers_modules::postgres::Postgres;

    static POSTGRES_CONTAINER: OnceLock<Arc<Container<Postgres>>> = OnceLock::new();
    pub fn run() {
        let container = POSTGRES_CONTAINER
            .get_or_init(|| {
                let container = Postgres::default().start().unwrap();
                let container = Arc::new(container);
                crate::CONTAINERS
                    .lock()
                    .unwrap()
                    .insert(TypeId::of::<Postgres>(), container.clone());
                container
            })
            .clone();
        let port = container
            .get_host_port_ipv4(5432)
            .expect("failed to get host port");
        unsafe {
            ::std::env::set_var(
                "DATABASE_URL",
                format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres"),
            );
        }
    }
}

#[cfg(feature = "mysql")]
mod mysql {
    use std::any::TypeId;
    use std::sync::{Arc, OnceLock};

    use testcontainers::{Container, runners::SyncRunner};
    use testcontainers_modules::mysql::Mysql;

    static MYSQL_CONTAINER: OnceLock<Arc<Container<Mysql>>> = OnceLock::new();
    pub fn run() {
        let container = MYSQL_CONTAINER
            .get_or_init(|| {
                let container = Mysql::default().start().unwrap();
                let container = Arc::new(container);
                crate::CONTAINERS
                    .lock()
                    .unwrap()
                    .insert(TypeId::of::<Mysql>(), container.clone());
                container
            })
            .clone();
        let port = container
            .get_host_port_ipv4(3306)
            .expect("failed to get host port");
        unsafe {
            ::std::env::set_var(
                "DATABASE_URL",
                format!("mysql://root@localhost:{port}/test"),
            );
        }
    }
}

#[cfg(feature = "postgres")]
pub use postgres::run as postgres;

#[cfg(feature = "mysql")]
pub use mysql::run as mysql;

#[cfg(feature = "mysql")]
pub use macros::mysql;
#[cfg(feature = "postgres")]
pub use macros::postgres;
