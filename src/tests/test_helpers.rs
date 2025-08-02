//! Test helper utilities for timeout management and progress reporting

use std::future::Future;
use std::time::Duration;
use tokio::time::timeout;

/// Default timeout for integration tests
#[allow(dead_code)]
pub const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Run a test with timeout and progress reporting
#[allow(dead_code)]
pub async fn run_test_with_timeout<F, T>(
    test_name: &str,
    future: F,
    timeout_duration: Option<Duration>,
) -> Result<T, String>
where
    F: Future<Output = T>,
{
    let duration = timeout_duration.unwrap_or(DEFAULT_TEST_TIMEOUT);

    println!(
        "[TEST: {}] Starting test with {}s timeout",
        test_name,
        duration.as_secs()
    );

    match timeout(duration, future).await {
        Ok(result) => {
            println!("[TEST: {test_name}] Completed successfully");
            Ok(result)
        }
        Err(_) => {
            let error_msg = format!(
                "[TEST: {}] TIMEOUT after {}s - test did not complete in time",
                test_name,
                duration.as_secs()
            );
            eprintln!("{error_msg}");
            Err(error_msg)
        }
    }
}

/// Print test progress message
#[allow(dead_code)]
pub fn test_progress(test_name: &str, message: &str) {
    println!("[TEST: {test_name}] {message}");
}

/// Macro for adding test timeouts
#[macro_export]
macro_rules! test_timeout {
    ($test_name:expr, $timeout_secs:expr, $test_body:expr) => {
        $crate::tests::test_helpers::run_test_with_timeout(
            $test_name,
            async { $test_body },
            Some(std::time::Duration::from_secs($timeout_secs)),
        )
        .await
        .expect("Test should complete within timeout")
    };
}

/// Macro for test progress logging
#[macro_export]
macro_rules! test_log {
    ($test_name:expr, $($arg:tt)*) => {
        println!("[TEST: {}] {}", $test_name, format!($($arg)*));
    };
}
