//! Secure storage for sensitive configuration values.
//!
//! This module provides optional secure storage for API keys and other
//! sensitive data using the system keychain when available.
//!
//! Platform support:
//! - **macOS**: Uses the macOS Keychain via Security Framework
//! - **Linux**: Uses Secret Service API (GNOME Keyring, KWallet, etc.)
//! - **Windows**: Uses Windows Credential Manager (wincred)
//! - **iOS**: Uses iOS Keychain Services
//! - **FreeBSD/OpenBSD**: Uses Secret Service if available

use anyhow::Result;

#[cfg(feature = "secure-storage")]
use keyring::Entry;

#[allow(dead_code)]
const SERVICE_NAME: &str = "rustycommit";

/// Store a secret securely in the system keyring.
///
/// Platform behavior:
/// - macOS: Stores in login keychain
/// - Linux: Stores in Secret Service (GNOME Keyring/KWallet)
/// - Windows: Stores in Windows Credential Manager
///
/// If the secure-storage feature is not enabled or the system doesn't
/// support keychain, this will return Ok(()) without storing anything.
pub fn store_secret(_key: &str, _value: &str) -> Result<()> {
    #[cfg(feature = "secure-storage")]
    {
        match Entry::new(SERVICE_NAME, _key) {
            Ok(entry) => {
                // Try to store, but don't fail if keyring is not available
                if let Err(e) = entry.set_password(_value) {
                    // Log the error for debugging but don't fail
                    eprintln!("Note: Could not store in secure storage: {}", e);
                }
            }
            Err(e) => {
                // Platform doesn't support keyring
                eprintln!("Note: Secure storage not available on this platform: {}", e);
            }
        }
    }
    Ok(())
}

/// Retrieve a secret from the system keyring.
///
/// Returns None if secure-storage is not enabled or the key doesn't exist.
pub fn get_secret(_key: &str) -> Result<Option<String>> {
    #[cfg(feature = "secure-storage")]
    {
        match Entry::new(SERVICE_NAME, _key) {
            Ok(entry) => match entry.get_password() {
                Ok(password) => Ok(Some(password)),
                Err(keyring::Error::NoEntry) => Ok(None),
                // Ignore other errors (e.g., no keychain available)
                Err(_) => Ok(None),
            },
            Err(_) => {
                // Platform doesn't support keyring
                Ok(None)
            }
        }
    }

    #[cfg(not(feature = "secure-storage"))]
    {
        Ok(None)
    }
}

/// Delete a secret from the system keyring.
///
/// If secure-storage is not enabled, this is a no-op.
pub fn delete_secret(_key: &str) -> Result<()> {
    #[cfg(feature = "secure-storage")]
    {
        match Entry::new(SERVICE_NAME, _key) {
            Ok(entry) => {
                // Try to delete, but don't fail if keyring is not available
                // In keyring v3, we use delete_credential() instead of delete_password()
                let _ = entry.delete_credential();
            }
            Err(_) => {
                // Platform doesn't support keyring - that's ok
            }
        }
    }

    Ok(())
}

/// Check if secure storage is available on this system.
///
/// Returns true only if the secure-storage feature is enabled AND
/// the system has a working keychain.
pub fn is_available() -> bool {
    #[cfg(feature = "secure-storage")]
    {
        // Try to create a test entry to see if keyring is available
        match Entry::new(SERVICE_NAME, "test") {
            Ok(entry) => {
                // Try to get a non-existent key - this should work if keyring is available
                matches!(entry.get_password(), Err(keyring::Error::NoEntry) | Ok(_))
            }
            Err(_) => false,
        }
    }

    #[cfg(not(feature = "secure-storage"))]
    {
        false
    }
}

/// Get detailed platform information for secure storage
pub fn get_platform_info() -> String {
    #[cfg(all(feature = "secure-storage", target_os = "macos"))]
    return "macOS Keychain".to_string();

    #[cfg(all(feature = "secure-storage", target_os = "linux"))]
    {
        // Try to detect which secret service is available
        if std::env::var("GNOME_KEYRING_CONTROL").is_ok() {
            return "GNOME Keyring".to_string();
        } else if std::env::var("KDE_FULL_SESSION").is_ok() {
            return "KWallet".to_string();
        } else {
            return "Linux Secret Service".to_string();
        }
    }

    #[cfg(all(feature = "secure-storage", target_os = "windows"))]
    return "Windows Credential Manager".to_string();

    #[cfg(all(feature = "secure-storage", target_os = "ios"))]
    return "iOS Keychain".to_string();

    #[cfg(all(feature = "secure-storage", target_os = "freebsd"))]
    return "FreeBSD Secret Service".to_string();

    #[cfg(all(feature = "secure-storage", target_os = "openbsd"))]
    return "OpenBSD Secret Service".to_string();

    #[cfg(not(feature = "secure-storage"))]
    return "Not compiled with secure storage support".to_string();

    // Fallback for unknown platforms with secure-storage enabled
    #[cfg(all(
        feature = "secure-storage",
        not(any(
            target_os = "macos",
            target_os = "linux",
            target_os = "windows",
            target_os = "ios",
            target_os = "freebsd",
            target_os = "openbsd"
        ))
    ))]
    return "Unknown platform".to_string();
}

/// Returns a user-friendly message about the secure storage status.
pub fn status_message() -> String {
    #[cfg(feature = "secure-storage")]
    {
        if is_available() {
            format!("Secure storage is available via {}", get_platform_info())
        } else {
            format!(
                "Secure storage feature is enabled but {} is not available",
                get_platform_info()
            )
        }
    }

    #[cfg(not(feature = "secure-storage"))]
    {
        "Secure storage is not enabled (compile with --features secure-storage to enable)"
            .to_string()
    }
}
