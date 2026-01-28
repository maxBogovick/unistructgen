# 🔒 Authentication Guide

This guide explains how to use authentication with UniStructGen's `struct_from_external_api!` macro.

## Overview

UniStructGen supports three authentication methods for accessing protected APIs at compile time:

1. **Bearer Token** - OAuth2, JWT, and other token-based authentication
2. **API Key** - Custom header-based authentication
3. **Basic Auth** - Username/password authentication

## Bearer Token Authentication

Bearer tokens are commonly used with OAuth2 and JWT authentication.

### Syntax

```rust
struct_from_external_api! {
    struct_name = "User",
    url_api = "https://api.example.com/user",
    auth_bearer = "your_token_here"
}
```

### Example

```rust
use unistructgen_macro::struct_from_external_api;

// GitHub API with personal access token
struct_from_external_api! {
    struct_name = "GitHubUser",
    url_api = "https://api.github.com/user",
    auth_bearer = "ghp_YourPersonalAccessToken123"
}
```

### Using Environment Variables

For sensitive tokens, use the `env!()` macro:

```rust
struct_from_external_api! {
    struct_name = "User",
    url_api = "https://api.example.com/user",
    auth_bearer = env!("API_TOKEN")
}
```

Set the environment variable before building:
```bash
export API_TOKEN="your_token_here"
cargo build
```

## API Key Authentication

API keys are sent in custom headers (commonly `X-API-Key`, `X-Auth-Token`, etc.).

### Syntax

```rust
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/data",
    auth_api_key = "Header-Name:value"
}
```

### Examples

```rust
// Standard X-API-Key header
struct_from_external_api! {
    struct_name = "WeatherData",
    url_api = "https://api.weather.com/current",
    auth_api_key = "X-API-Key:abc123xyz789"
}

// Custom header name
struct_from_external_api! {
    struct_name = "Analytics",
    url_api = "https://api.analytics.com/stats",
    auth_api_key = "X-Auth-Token:secret_token_456"
}

// With environment variable
struct_from_external_api! {
    struct_name = "CloudData",
    url_api = "https://api.cloud.com/resources",
    auth_api_key = concat!("X-API-Key:", env!("CLOUD_API_KEY"))
}
```

## Basic Authentication

HTTP Basic Authentication encodes username and password in base64.

### Syntax

```rust
struct_from_external_api! {
    struct_name = "Resource",
    url_api = "https://api.example.com/resource",
    auth_basic = "username:password"
}
```

### Examples

```rust
// Simple basic auth
struct_from_external_api! {
    struct_name = "ProtectedData",
    url_api = "https://api.example.com/data",
    auth_basic = "admin:secret123"
}

// With environment variables
struct_from_external_api! {
    struct_name = "SecureResource",
    url_api = "https://api.example.com/secure",
    auth_basic = concat!(env!("API_USER"), ":", env!("API_PASS"))
}
```

⚠️ **Security Warning**: Avoid hardcoding credentials in source code. Always use environment variables for sensitive information.

## Complete Example

Here's a comprehensive example using all authentication methods:

```rust
use unistructgen_macro::struct_from_external_api;

// Public API (no authentication)
struct_from_external_api! {
    struct_name = "PublicTodo",
    url_api = "https://jsonplaceholder.typicode.com/todos/1"
}

// Bearer token authentication
struct_from_external_api! {
    struct_name = "GitHubRepo",
    url_api = "https://api.github.com/repos/rust-lang/rust",
    auth_bearer = env!("GITHUB_TOKEN")
}

// API key authentication
struct_from_external_api! {
    struct_name = "WeatherData",
    url_api = "https://api.openweathermap.org/data/2.5/weather?q=London",
    auth_api_key = concat!("X-API-Key:", env!("WEATHER_API_KEY"))
}

// Basic authentication
struct_from_external_api! {
    struct_name = "PrivateResource",
    url_api = "https://private-api.example.com/data",
    auth_basic = concat!(env!("API_USER"), ":", env!("API_PASS"))
}

fn main() {
    // Use your generated structs with full type safety!
    println!("All structs generated successfully!");
}
```

## Combining with Other Options

Authentication can be combined with other macro options:

```rust
struct_from_external_api! {
    struct_name = "CompleteExample",
    url_api = "https://api.example.com/data",

    // Authentication
    auth_bearer = env!("API_TOKEN"),

    // Request options
    timeout = 30000,        // 30 second timeout
    method = "GET",         // HTTP method

    // Generation options
    max_depth = 5,          // Limit nesting depth
    serde = true,           // Add serde derives (default)
    default = true,         // Add Default derive
    optional = false,       // Don't make fields Option<T>
}
```

## Security Best Practices

### ✅ DO:

- Use environment variables for sensitive credentials
- Rotate API keys regularly
- Use separate tokens for development and production
- Set appropriate token permissions/scopes
- Use HTTPS endpoints only

### ❌ DON'T:

- Hardcode credentials in source code
- Commit credentials to version control
- Share API keys in public repositories
- Use the same credentials across environments

## Environment Variable Setup

### Development

Create a `.env` file (add to `.gitignore`):

```bash
# .env
API_TOKEN=your_development_token
GITHUB_TOKEN=ghp_YourDevToken123
WEATHER_API_KEY=abc123xyz789
API_USER=dev_user
API_PASS=dev_password
```

Load in your build:
```toml
# Cargo.toml
[build-dependencies]
dotenv = "0.15"
```

### CI/CD

Set environment variables in your CI configuration:

**GitHub Actions:**
```yaml
env:
  API_TOKEN: ${{ secrets.API_TOKEN }}
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**GitLab CI:**
```yaml
variables:
  API_TOKEN: $API_TOKEN
```

## Troubleshooting

### "Missing environment variable" error

```
error: environment variable `API_TOKEN` not defined at compile time
```

**Solution**: Set the environment variable before building:
```bash
export API_TOKEN="your_token"
cargo build
```

### Authentication fails

1. Verify token is valid and not expired
2. Check API endpoint requires the authentication method you're using
3. Ensure header names match API requirements (case-sensitive)
4. Verify token has required permissions/scopes

### Timeout errors

Increase the timeout for slow APIs:
```rust
struct_from_external_api! {
    struct_name = "SlowAPI",
    url_api = "https://slow-api.example.com/data",
    auth_bearer = env!("API_TOKEN"),
    timeout = 60000  // 60 seconds
}
```

## Advanced: Custom Headers

While not authentication per se, you can use API key method for any custom header:

```rust
// Custom user agent
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/data",
    auth_api_key = "User-Agent:MyApp/1.0"
}

// Custom accept header
struct_from_external_api! {
    struct_name = "JsonData",
    url_api = "https://api.example.com/data",
    auth_api_key = "Accept:application/json"
}
```

## Related Documentation

- [Quick Start Guide](QUICKSTART.md)
- [Complete Documentation](DOCUMENTATION.md)
- [API Reference](https://docs.rs/unistructgen)

---

**Need help?** Open an issue on [GitHub](https://github.com/maxBogovick/unistructgen/issues)
