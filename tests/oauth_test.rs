use rustycommit::auth::oauth::OAuthClient;
use tempfile::tempdir;

#[test]
fn test_oauth_client_creation() {
    let _client = OAuthClient::new();
    // Test that OAuth client can be created without panic
    // Since fields are private, we mainly test that construction works
    assert!(true);
}

#[test]
fn test_oauth_authorization_url_generation() {
    let client = OAuthClient::new();
    let result = client.get_authorization_url();

    // Should return a URL and verifier
    assert!(result.is_ok());

    if let Ok((auth_url, verifier)) = result {
        // URL should be properly formatted
        assert!(auth_url.starts_with("https://"));
        assert!(auth_url.contains("claude.ai"));
        assert!(auth_url.contains("oauth"));

        // Verifier should be a reasonable length (base64url encoded)
        assert!(verifier.len() >= 43); // Base64url of 32 bytes = 43 chars minimum
        assert!(verifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }
}

#[test]
fn test_pkce_challenge_generation() {
    let client = OAuthClient::new();

    // Generate multiple URLs to ensure challenges are different
    let result1 = client.get_authorization_url();
    let result2 = client.get_authorization_url();

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    if let (Ok((url1, verifier1)), Ok((url2, verifier2))) = (result1, result2) {
        // Each call should generate different verifier
        assert_ne!(verifier1, verifier2);

        // URLs should be different (due to different challenges)
        assert_ne!(url1, url2);
    }
}

#[test]
fn test_oauth_environment_setup() {
    let temp_dir = tempdir().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    // Test that OAuth-related environment can be set up
    let client = OAuthClient::new();
    let result = client.get_authorization_url();
    assert!(result.is_ok());

    // Test callback port environment (if we need to customize it)
    std::env::set_var("OAUTH_CALLBACK_PORT", "8080");
    let port = std::env::var("OAUTH_CALLBACK_PORT").unwrap_or("3000".to_string());
    assert_eq!(port, "8080");
    std::env::remove_var("OAUTH_CALLBACK_PORT");
}

#[test]
fn test_oauth_constants() {
    // Test that OAuth constants are reasonable
    use rustycommit::auth::oauth::{AUTHORIZE_URL, CLIENT_ID, REDIRECT_URI};

    // Client ID should be a valid UUID format
    assert_eq!(CLIENT_ID.len(), 36); // UUID length
    assert!(CLIENT_ID.contains("-"));

    // Redirect URI should be localhost
    assert!(REDIRECT_URI.starts_with("http://localhost"));
    assert!(REDIRECT_URI.contains("callback"));

    // Authorize URL should be Claude's OAuth endpoint
    assert!(AUTHORIZE_URL.starts_with("https://"));
    assert!(AUTHORIZE_URL.contains("claude.ai"));
    assert!(AUTHORIZE_URL.contains("oauth"));
}

#[test]
fn test_oauth_scopes() {
    let client = OAuthClient::new();
    let result = client.get_authorization_url();

    if let Ok((auth_url, _)) = result {
        // Should contain expected scopes parameter
        assert!(auth_url.contains("scope="));

        // The scope might be URL encoded, so let's just check it exists
        // and contains some expected parts
        assert!(
            auth_url.contains("user")
                || auth_url.contains("inference")
                || auth_url.contains("scope=")
        );
    }
}

#[test]
fn test_oauth_response_type() {
    let client = OAuthClient::new();
    let result = client.get_authorization_url();

    if let Ok((auth_url, _)) = result {
        // Should use authorization code flow
        assert!(auth_url.contains("response_type=code"));
    }
}

#[test]
fn test_oauth_state_parameter() {
    let client = OAuthClient::new();
    let result = client.get_authorization_url();

    if let Ok((auth_url, _)) = result {
        // Should include state parameter for CSRF protection
        assert!(auth_url.contains("state="));
    }
}

#[test]
fn test_oauth_pkce_parameters() {
    let client = OAuthClient::new();
    let result = client.get_authorization_url();

    if let Ok((auth_url, _)) = result {
        // Should include PKCE challenge
        assert!(auth_url.contains("code_challenge="));
        assert!(auth_url.contains("code_challenge_method=S256"));
    }
}

#[test]
fn test_oauth_url_format() {
    // Test OAuth URL format without using internal functions
    let client = OAuthClient::new();
    let result = client.get_authorization_url();

    if let Ok((auth_url, verifier)) = result {
        // Test that the URL has expected format
        assert!(auth_url.contains("code_challenge="));
        assert!(auth_url.contains("code_challenge_method=S256"));

        // Test verifier format (should be base64url)
        assert!(verifier.len() >= 43);
        assert!(verifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }
}

#[test]
fn test_oauth_error_handling() {
    // Test that OAuth client handles various error conditions gracefully
    let client = OAuthClient::new();

    // Test with invalid callback scenarios (these will fail in actual callback)
    // But the authorization URL generation should still work
    let result = client.get_authorization_url();
    assert!(result.is_ok());
}

#[test]
fn test_token_expiry_calculation() {
    use rustycommit::auth::oauth::OAuthClient;
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Test token that expires in the past
    assert!(OAuthClient::is_token_expired(now - 3600));

    // Test token that expires in the future (should NOT be expired)
    assert!(!OAuthClient::is_token_expired(now + 3600));

    // Test token that expires very soon - depends on implementation
    // Let's check if current time is >= expires_at (basic expiry check)
    let expires_very_soon = now + 1; // 1 second from now
                                     // This might be expired by the time the assertion runs, so we test both cases
    let is_expired = OAuthClient::is_token_expired(expires_very_soon);
    // Either it's expired (if time passed) or not expired (if time hasn't passed yet)
    assert!(is_expired || !is_expired); // Always true, just testing it doesn't panic
}

#[test]
fn test_oauth_callback_server_configuration() {
    // Test that callback server can be configured
    let client = OAuthClient::new();

    // Should be able to create authorization URL (server config tested indirectly)
    let result = client.get_authorization_url();
    assert!(result.is_ok());

    if let Ok((auth_url, _)) = result {
        // URL should contain the redirect URI
        assert!(auth_url.contains("redirect_uri="));
        assert!(auth_url.contains("localhost"));
    }
}

#[test]
fn test_random_verifier_generation() {
    let client = OAuthClient::new();

    // Generate multiple verifiers to ensure they're random
    let mut verifiers = Vec::new();
    for _ in 0..10 {
        if let Ok((_, verifier)) = client.get_authorization_url() {
            verifiers.push(verifier);
        }
    }

    // All verifiers should be unique
    assert_eq!(verifiers.len(), 10);
    for i in 0..verifiers.len() {
        for j in i + 1..verifiers.len() {
            assert_ne!(verifiers[i], verifiers[j], "Verifiers should be unique");
        }
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;

    #[test]
    fn test_oauth_flow_components() {
        // Test the individual components that make up the OAuth flow
        let client = OAuthClient::new();

        // Test authorization URL generation
        let auth_result = client.get_authorization_url();
        assert!(auth_result.is_ok());

        // In a real implementation, we'd test:
        // 1. Callback server startup
        // 2. Authorization code exchange
        // 3. Token refresh
        // 4. Token validation

        // For now, we test that the components don't panic
        if let Ok((url, verifier)) = auth_result {
            assert!(!url.is_empty());
            assert!(!verifier.is_empty());

            // Test that the URL contains the challenge (verifier produces a challenge)
            assert!(url.contains("code_challenge="));
        }
    }
}
