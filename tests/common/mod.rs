//! Common test utilities for DLsite tests
//!
//! This module provides:
//! - Fixture loading helpers
//! - Test retry configuration (zero-delay for fast tests)

use std::fs;
use std::path::Path;
use std::time::Duration;

use dlsite_rs::retry::RetryConfig;

/// Load a fixture file from the tests/fixtures directory
///
/// # Arguments
/// * `path` - Relative path from tests/fixtures/ (e.g., "public/product_api/RJ403038.json")
///
/// # Returns
/// The contents of the fixture file as a String
#[allow(dead_code)]
pub fn load_fixture(path: &str) -> String {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path);

    fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to load fixture '{}': {}", fixture_path.display(), e))
}

/// Load a fixture file and parse as JSON
///
/// # Arguments
/// * `path` - Relative path from tests/fixtures/
///
/// # Returns
/// The parsed JSON value
#[allow(dead_code)]
pub fn load_json_fixture(path: &str) -> serde_json::Value {
    let content = load_fixture(path);
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse JSON fixture '{}': {}", path, e))
}

/// Create a retry config with zero delays for fast tests
///
/// This configuration allows testing retry behavior without waiting
/// for actual delays between retries.
pub fn test_retry_config() -> RetryConfig {
    RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_millis(0),
        max_delay: Duration::from_millis(0),
        backoff_multiplier: 1.0,
    }
}

/// Check if live tests are enabled via environment variable
pub fn live_tests_enabled() -> bool {
    std::env::var("DLSITE_LIVE_TESTS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_retry_config_has_zero_delay() {
        let config = test_retry_config();
        assert_eq!(config.initial_delay, Duration::from_millis(0));
        assert_eq!(config.max_delay, Duration::from_millis(0));
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_live_tests_disabled_by_default() {
        // This test assumes DLSITE_LIVE_TESTS is not set
        // In CI, this should pass
        assert!(!live_tests_enabled());
    }
}
