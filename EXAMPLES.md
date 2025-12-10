# UniStructGen Examples

This document provides practical examples of using UniStructGen.

## Basic Usage

### Simple JSON to Struct

Input file `person.json`:
```json
{
  "name": "John Doe",
  "age": 30,
  "active": true
}
```

Command:
```bash
unistructgen generate --input person.json --name Person
```

Output:
```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Person {
    pub active: bool,
    pub age: i64,
    pub name: String,
}
```

### Nested Structures

Input file `company.json`:
```json
{
  "name": "Acme Corp",
  "founded": 2010,
  "address": {
    "street": "123 Main St",
    "city": "New York",
    "zip": "10001"
  }
}
```

Command:
```bash
unistructgen generate --input company.json --name Company
```

Output:
```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Address {
    pub city: String,
    pub street: String,
    pub zip: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Company {
    pub address: Address,
    pub founded: i64,
    pub name: String,
}
```

### Arrays and Collections

Input file `blog.json`:
```json
{
  "title": "My Blog Post",
  "tags": ["rust", "programming", "tutorial"],
  "author": "Alice",
  "comments": [
    {
      "user": "Bob",
      "message": "Great post!"
    }
  ]
}
```

Command:
```bash
unistructgen generate --input blog.json --name BlogPost
```

Output:
```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CommentsItem {
    pub message: String,
    pub user: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BlogPost {
    pub author: String,
    pub comments: Vec<CommentsItem>,
    pub tags: Vec<String>,
    pub title: String,
}
```

### Special Type Detection

UniStructGen automatically detects and uses appropriate Rust types:

#### DateTime Detection
```json
{
  "event": "Meeting",
  "scheduled_at": "2024-12-09T14:30:00Z"
}
```

Generates:
```rust
pub scheduled_at: chrono::DateTime<chrono::Utc>,
```

#### UUID Detection
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Item"
}
```

Generates:
```rust
pub id: uuid::Uuid,
```

### Field Name Sanitization

UniStructGen automatically converts field names to Rust conventions:

Input:
```json
{
  "firstName": "John",
  "last_name": "Doe",
  "user-id": 123,
  "IsActive": true
}
```

Output:
```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct User {
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "IsActive")]
    pub is_active: bool,
    pub last_name: String,
    #[serde(rename = "user-id")]
    pub user_id: i64,
}
```

### Optional Fields

Use the `--optional` flag to make all fields optional:

Command:
```bash
unistructgen generate --input person.json --name Person --optional
```

Output:
```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Person {
    pub active: Option<bool>,
    pub age: Option<i64>,
    pub name: Option<String>,
}
```

### Without Serde

Generate plain structs without serde:

Command:
```bash
unistructgen generate --input person.json --name Person --serde false
```

Output:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    pub active: bool,
    pub age: i64,
    pub name: String,
}
```

### With Default Derive

Add Default trait implementation:

Command:
```bash
unistructgen generate --input person.json --name Person --default true
```

Output:
```rust
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Person {
    pub active: bool,
    pub age: i64,
    pub name: String,
}
```

## Integration Examples

### In Build Scripts

Create `build.rs`:
```rust
use std::process::Command;

fn main() {
    // Generate structs during build
    Command::new("unistructgen")
        .args(&[
            "generate",
            "--input", "schema/user.json",
            "--output", "src/generated/user.rs",
            "--name", "User"
        ])
        .status()
        .expect("Failed to generate structs");

    println!("cargo:rerun-if-changed=schema/user.json");
}
```

### Using Generated Code

```rust
// Import the generated types
mod generated {
    include!("generated/user.rs");
}

use generated::User;

fn main() {
    let json = r#"{"id": 1, "name": "Alice", "age": 30}"#;
    let user: User = serde_json::from_str(json).unwrap();
    println!("{:?}", user);
}
```

## Real-World Example: API Response

Input `api_response.json`:
```json
{
  "success": true,
  "data": {
    "users": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "username": "alice",
        "email": "alice@example.com",
        "created_at": "2024-01-15T10:30:00Z",
        "profile": {
          "avatar_url": "https://example.com/avatar.jpg",
          "bio": "Software Engineer",
          "followers": 150
        }
      }
    ]
  },
  "meta": {
    "total": 100,
    "page": 1,
    "per_page": 10
  }
}
```

Command:
```bash
unistructgen generate --input api_response.json --name ApiResponse --output src/models/api.rs
```

This generates complete, ready-to-use Rust types for the entire API response structure!
