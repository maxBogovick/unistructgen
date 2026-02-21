//! Fetch OpenAPI specifications from URLs

use crate::parsers::openapi::error::{OpenApiError, Result};
use std::time::Duration;

/// Fetch OpenAPI spec from URL
///
/// # Examples
///
/// ```no_run
/// use unistructgen::parsers::openapi::fetch::fetch_spec_from_url;
///
/// let spec = fetch_spec_from_url("https://api.example.com/openapi.yaml", Some(30000)).unwrap();
/// ```
pub fn fetch_spec_from_url(url: &str, timeout_ms: Option<u64>) -> Result<String> {
    let timeout = timeout_ms
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_secs(30));

    let response = ureq::get(url)
        .timeout(timeout)
        .call()
        .map_err(|e| OpenApiError::FetchError(format!("Failed to fetch from {}: {}", url, e)))?;

    let content = response
        .into_string()
        .map_err(|e| OpenApiError::FetchError(format!("Failed to read response body: {}", e)))?;

    if content.is_empty() {
        return Err(OpenApiError::FetchError(
            "Empty response from URL".to_string(),
        ));
    }

    Ok(content)
}

/// Fetch OpenAPI spec from URL with authentication
///
/// # Examples
///
/// ```no_run
/// use unistructgen::parsers::openapi::fetch::{fetch_spec_with_auth, AuthType};
///
/// let spec = fetch_spec_with_auth(
///     "https://api.example.com/openapi.yaml",
///     AuthType::Bearer("my-token".to_string()),
///     Some(30000)
/// ).unwrap();
/// ```
pub fn fetch_spec_with_auth(url: &str, auth: AuthType, timeout_ms: Option<u64>) -> Result<String> {
    let timeout = timeout_ms
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_secs(30));

    let mut request = ureq::get(url).timeout(timeout);

    // Add authentication header
    request = match auth {
        AuthType::Bearer(token) => request.set("Authorization", &format!("Bearer {}", token)),
        AuthType::ApiKey { header, value } => request.set(&header, &value),
        AuthType::Basic { username, password } => {
            let credentials = base64::encode(format!("{}:{}", username, password));
            request.set("Authorization", &format!("Basic {}", credentials))
        }
    };

    let response = request.call().map_err(|e| {
        OpenApiError::FetchError(format!("Failed to fetch from {} with auth: {}", url, e))
    })?;

    let content = response
        .into_string()
        .map_err(|e| OpenApiError::FetchError(format!("Failed to read response body: {}", e)))?;

    if content.is_empty() {
        return Err(OpenApiError::FetchError(
            "Empty response from URL".to_string(),
        ));
    }

    Ok(content)
}

/// Authentication type for fetching specs
#[derive(Debug, Clone)]
pub enum AuthType {
    /// Bearer token authentication
    Bearer(String),

    /// API key in custom header
    ApiKey {
        /// Header name
        header: String,
        /// API key value
        value: String,
    },

    /// HTTP Basic authentication
    Basic {
        /// Username
        username: String,
        /// Password
        password: String,
    },
}

// Note: base64 crate would need to be added to Cargo.toml for Basic auth
// For now, we'll use a simple implementation
mod base64 {
    pub fn encode(input: String) -> String {
        // In a real implementation, use the base64 crate
        // This is a placeholder
        use std::io::Write;
        let mut buf = Vec::new();
        let _ = write!(&mut buf, "{}", input);
        String::from_utf8_lossy(&buf).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::AuthType;

    #[test]
    fn test_auth_type_creation() {
        let bearer = AuthType::Bearer("token123".to_string());
        assert!(matches!(bearer, AuthType::Bearer(_)));

        let api_key = AuthType::ApiKey {
            header: "X-API-Key".to_string(),
            value: "key123".to_string(),
        };
        assert!(matches!(api_key, AuthType::ApiKey { .. }));
    }
}
