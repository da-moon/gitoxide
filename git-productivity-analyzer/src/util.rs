use crate::error::Result;
use miette::IntoDiagnostic;
use tokio::task;

pub async fn spawn_blocking<F, T>(func: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    task::spawn_blocking(func).await.into_diagnostic()?
}
