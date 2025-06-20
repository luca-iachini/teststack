#![doc = include_str!("../../README.md")]

use ctor::{ctor, dtor};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::future::Future;
use std::thread;
use std::{any::TypeId, ops::Deref, pin::Pin, sync::Arc};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

pub use testcontainers::core::ContainerPort;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ContainerRequest, Image};

mod custom;
mod db;

#[derive(Clone)]
pub struct GenericContainer(Arc<dyn RunningContainer>);

static CONTAINERS: Lazy<DashMap<TypeId, Mutex<Option<GenericContainer>>>> = Lazy::new(DashMap::new);

/// Start a container and return a handle to it.
pub async fn container<I: Image + 'static>(
    request: impl Into<ContainerRequest<I>> + AsyncRunner<I>,
) -> GenericContainer {
    let ty = TypeId::of::<I>();
    let entry = CONTAINERS.entry(ty).or_default();
    let mut guard = entry.lock().await;
    if let Some(container) = guard.as_ref() {
        container.clone()
    } else {
        let container = GenericContainer(Arc::new(request.start().await.unwrap()));
        *guard = Some(container.clone());
        container
    }
}

/// Start a stack of containers.
pub async fn stack<I: Image + 'static>(
    containers: Vec<impl Into<ContainerRequest<I>> + AsyncRunner<I> + Send + Sync + 'static>,
) -> Vec<GenericContainer> {
    let mut set = JoinSet::new();
    for request in containers {
        set.spawn(container(request));
    }
    set.join_all().await
}

pub trait RunningContainer: Send + Sync {
    /// Get the docker container ID.
    fn id(&self) -> &str;
    /// Get the host port for a given internal port.
    fn get_host_port_ipv4<'a>(
        &'a self,
        internal_port: ContainerPort,
    ) -> Pin<Box<dyn Future<Output = Result<u16, testcontainers::TestcontainersError>> + Send + 'a>>;
}

impl<I: Image> RunningContainer for ContainerAsync<I> {
    fn id(&self) -> &str {
        self.id()
    }
    fn get_host_port_ipv4<'a>(
        &'a self,
        internal_port: ContainerPort,
    ) -> Pin<Box<dyn Future<Output = Result<u16, testcontainers::TestcontainersError>> + Send + 'a>>
    {
        Box::pin(self.get_host_port_ipv4(internal_port))
    }
}

impl Deref for GenericContainer {
    type Target = dyn RunningContainer;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// A running container with configurations.
pub struct TestContainer<T> {
    container: GenericContainer,
    pub conf: T,
}

impl<T> Deref for TestContainer<T> {
    type Target = GenericContainer;
    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

/// A trait to initialize the arguments to pass to the test function.
pub trait Init<T>: Sized {
    fn init(self) -> impl Future<Output = T>;
}

impl<T, C> Init<T> for TestContainer<C>
where
    T: From<TestContainer<C>>,
{
    async fn init(self) -> T {
        self.into()
    }
}

#[ctor]
fn init() {
    Lazy::force(&CONTAINERS);
    Lazy::force(&REMOVE_CONTAINERS);
}

#[dtor]
fn remove_containers() {
    let (tx, mut rx) = mpsc::channel(1);
    let _ = REMOVE_CONTAINERS.blocking_send(tx);
    rx.blocking_recv();
}

/// Remove all containers when the test harness exits.
static REMOVE_CONTAINERS: Lazy<mpsc::Sender<mpsc::Sender<()>>> = Lazy::new(|| {
    let (tx, mut rx) = mpsc::channel::<mpsc::Sender<()>>(1);
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            tokio::select! {
                Some(reply_tx) = rx.recv() => {
                    CONTAINERS.clear();
                    let _ = reply_tx.send(()).await;
                },
                _ = tokio::signal::ctrl_c() => {
                    CONTAINERS.clear();
                }
            }
        });
    });
    tx
});

#[cfg(feature = "macros")]
pub use teststack_macros::stack;

pub use custom::{run as custom, CustomContainer};

#[cfg(feature = "mysql")]
pub use db::mysql::run as mysql;
#[cfg(feature = "postgres")]
pub use db::postgres::run as postgres;

pub use db::{DbConf, DbContainer, DbName};
