use std::{future::Future, pin::Pin};

use crate::executor::Executor;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TokioExecutor;

impl Executor for TokioExecutor {
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        tokio::task::spawn(future);
    }
}
