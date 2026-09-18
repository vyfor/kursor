use std::{future::Future, pin::Pin, sync::Arc};

#[cfg(feature = "tokio")]
pub mod tokio;
#[cfg(feature = "tokio")]
pub use self::tokio::TokioExecutor;

pub trait Executor: Send + Sync + 'static {
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>);
}

impl<F> Executor for F
where
    F: Fn(Pin<Box<dyn Future<Output = ()> + Send>>) + Send + Sync + 'static,
{
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        (self)(future);
    }
}

#[derive(Clone)]
pub struct TaskExecutor(pub Arc<dyn Executor>);

impl TaskExecutor {
    pub fn new(executor: impl Executor) -> Self {
        Self(Arc::new(executor))
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.0.spawn(Box::pin(future));
    }
}

impl PartialEq for TaskExecutor {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
