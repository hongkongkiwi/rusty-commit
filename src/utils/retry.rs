use anyhow::Result;
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use backoff::tokio::retry;
use std::time::Duration;

/// Determines if an error is retryable
pub fn is_retryable_error(error: &anyhow::Error) -> bool {
    let error_msg = error.to_string().to_lowercase();
    
    // Retryable errors: network issues, timeouts, rate limits, server errors
    error_msg.contains("429") ||  // Rate limit
    error_msg.contains("rate_limit") ||
    error_msg.contains("rate limit") ||
    error_msg.contains("500") ||  // Internal server error
    error_msg.contains("502") ||  // Bad gateway
    error_msg.contains("503") ||  // Service unavailable
    error_msg.contains("504") ||  // Gateway timeout
    error_msg.contains("timeout") ||
    error_msg.contains("connection") ||
    error_msg.contains("network") ||
    error_msg.contains("dns") ||
    error_msg.contains("overloaded")
}

/// Determines if an error is permanent (should not retry)
pub fn is_permanent_error(error: &anyhow::Error) -> bool {
    let error_msg = error.to_string().to_lowercase();
    
    // Permanent errors: auth issues, invalid requests, quota exceeded
    error_msg.contains("401") ||  // Unauthorized
    error_msg.contains("403") ||  // Forbidden
    error_msg.contains("invalid_api_key") ||
    error_msg.contains("insufficient_quota") ||
    error_msg.contains("quota exceeded") ||
    error_msg.contains("invalid request") ||
    error_msg.contains("model not found") ||
    error_msg.contains("400")     // Bad request
}

/// Create a backoff policy for API retries
pub fn create_backoff() -> ExponentialBackoff {
    ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_millis(500))
        .with_max_interval(Duration::from_secs(30))
        .with_multiplier(2.0)
        .with_max_elapsed_time(Some(Duration::from_secs(120))) // 2 minutes total
        .build()
}

/// Retry an async operation with exponential backoff
pub async fn retry_async<F, Fut, T>(operation: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let backoff = create_backoff();
    
    retry(backoff, || async {
        match operation().await {
            Ok(result) => Ok(result),
            Err(error) => {
                if is_permanent_error(&error) {
                    // Don't retry permanent errors
                    Err(backoff::Error::permanent(error))
                } else if is_retryable_error(&error) {
                    // Retry transient errors
                    Err(backoff::Error::transient(error))
                } else {
                    // Unknown errors - treat as permanent to be safe
                    Err(backoff::Error::permanent(error))
                }
            }
        }
    }).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_is_retryable_error() {
        assert!(is_retryable_error(&anyhow!("429 Rate limit exceeded")));
        assert!(is_retryable_error(&anyhow!("500 Internal server error")));
        assert!(is_retryable_error(&anyhow!("Connection timeout")));
        assert!(is_retryable_error(&anyhow!("Network error")));
        assert!(is_retryable_error(&anyhow!("Model overloaded")));
        
        assert!(!is_retryable_error(&anyhow!("401 Unauthorized")));
        assert!(!is_retryable_error(&anyhow!("Invalid API key")));
    }

    #[test]
    fn test_is_permanent_error() {
        assert!(is_permanent_error(&anyhow!("401 Unauthorized")));
        assert!(is_permanent_error(&anyhow!("Invalid API key")));
        assert!(is_permanent_error(&anyhow!("Insufficient quota")));
        assert!(is_permanent_error(&anyhow!("400 Bad request")));
        
        assert!(!is_permanent_error(&anyhow!("429 Rate limit")));
        assert!(!is_permanent_error(&anyhow!("500 Server error")));
    }

    #[tokio::test]
    async fn test_retry_success() {
        let mut attempts = 0;
        
        let result = retry_async(|| async {
            attempts += 1;
            if attempts < 3 {
                Err(anyhow!("429 Rate limit"))
            } else {
                Ok("success".to_string())
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts, 3);
    }

    #[tokio::test]
    async fn test_retry_permanent_error() {
        let mut attempts = 0;
        
        let result = retry_async(|| async {
            attempts += 1;
            Err(anyhow!("401 Unauthorized"))
        }).await;
        
        assert!(result.is_err());
        assert_eq!(attempts, 1); // Should not retry permanent errors
    }
}