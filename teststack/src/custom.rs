use testcontainers::{runners::AsyncRunner, ContainerRequest, Image};

use crate::TestContainer;

/// Run a custom container with the given configuration.
pub async fn run<I: Image + 'static>(
    request: impl Into<ContainerRequest<I>> + AsyncRunner<I>,
) -> CustomContainer {
    let container = crate::container(request).await;
    CustomContainer {
        container,
        conf: (),
    }
}

/// A custom container with no specific configuration.
pub type CustomContainer = TestContainer<()>;
