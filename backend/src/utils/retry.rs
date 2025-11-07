use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

/// 带重试的异步操作
pub async fn retry_async<F, Fut, T, E>(
    mut operation: F,
    max_retries: u32,
    delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempts = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts >= max_retries {
                    return Err(e);
                }
                warn!(
                    "Operation failed (attempt {}/{}): {}. Retrying...",
                    attempts, max_retries, e
                );
                sleep(delay * attempts).await;
            }
        }
    }
}
