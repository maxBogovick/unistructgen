# 🔮 UniStructGen Proc Macros

**Compile-time генерация Rust-структур с нулевым runtime overhead**

[![Crate](https://img.shields.io/crates/v/unistructgen-macro.svg)](https://crates.io/crates/unistructgen-macro)
[![Docs](https://docs.rs/unistructgen-macro/badge.svg)](https://docs.rs/unistructgen-macro)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../LICENSE-MIT)

---

## 📋 Содержание

- [Обзор](#-обзор)
- [Установка](#-установка)
- [Макросы](#-макросы)
- [generate_struct_from_json!](#-generate_struct_from_json)
- [#[json_struct]](#-json_struct)
- [struct_from_external_api!](#-struct_from_external_api)
- [openapi_to_rust!](#-openapi_to_rust)
- [Аутентификация](#-аутентификация)
- [Compile-time Fetch Controls](#-compile-time-fetch-controls)
- [Type Inference](#-type-inference)
- [Примеры](#-примеры)

---

## 🎯 Обзор

`unistructgen-macro` предоставляет процедурные макросы для генерации Rust-кода во время компиляции:

### Преимущества compile-time генерации

| Аспект | Compile-time | Runtime |
|--------|--------------|---------|
| **Производительность** | Zero overhead | Parsing cost |
| **Type Safety** | Полная проверка | Частичная |
| **IDE Support** | Полный autocomplete | Ограниченный |
| **Error Detection** | При компиляции | При выполнении |

### Доступные макросы

| Макрос | Источник | Описание |
|--------|----------|----------|
| `generate_struct_from_json!` | Inline JSON | Генерация из JSON строки |
| `#[json_struct]` | Const string | Атрибут для const |
| `struct_from_external_api!` | HTTP API | Загрузка JSON из API |
| `openapi_to_rust!` | OpenAPI spec | Полная генерация из OpenAPI |

---

## 📦 Установка

```toml
[dependencies]
unistructgen-macro = "0.1"

# Для работы с сгенерированным кодом
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Для специальных типов (опционально)
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["serde", "v4"] }
```

---

## 🔧 generate_struct_from_json!

Генерирует Rust-структуры из inline JSON.

### Синтаксис

```rust
generate_struct_from_json! {
    name = "StructName",
    json = r#"{ "field": "value" }"#,
    // Опциональные параметры:
    serde = true,      // Добавить serde derives (default: true)
    default = false,   // Добавить Default derive (default: false)
    optional = false,  // Все поля как Option<T> (default: false)
}
```

### Примеры

**Базовое использование:**

```rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "User",
    json = r#"{
        "id": 1,
        "name": "Alice",
        "email": "alice@example.com"
    }"#
}

fn main() {
    let user = User {
        id: 42,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    let json = serde_json::to_string(&user).unwrap();
    println!("{}", json);
}
```

**С вложенными объектами:**

```rust
generate_struct_from_json! {
    name = "Order",
    json = r#"{
        "id": "ORD-001",
        "customer": {
            "name": "Alice",
            "email": "alice@example.com"
        },
        "items": [
            { "product": "Widget", "quantity": 2, "price": 29.99 }
        ],
        "total": 59.98
    }"#
}

// Генерирует:
// - Customer { name: String, email: String }
// - ItemsItem { product: String, quantity: i64, price: f64 }
// - Order { id: String, customer: Customer, items: Vec<ItemsItem>, total: f64 }
```

**С Default:**

```rust
generate_struct_from_json! {
    name = "Config",
    json = r#"{
        "host": "localhost",
        "port": 8080,
        "debug": false
    }"#,
    default = true
}

let config = Config::default();
```

---

## 🏷️ #[json_struct]

Атрибут-макрос для генерации из const-строки.

### Синтаксис

```rust
#[json_struct(name = "StructName", serde = true, default = false)]
const SAMPLE: &str = r#"{ "json": "data" }"#;
```

### Пример

```rust
use unistructgen_macro::json_struct;

#[json_struct(name = "Settings", default = true)]
const SETTINGS_SAMPLE: &str = r#"{
    "theme": "dark",
    "font_size": 14,
    "notifications": true
}"#;

fn main() {
    let settings = Settings::default();
    println!("Theme: {}", settings.theme);
}
```

---

## 🌐 struct_from_external_api!

Загружает JSON из внешнего API во время компиляции.

### Синтаксис

```rust
struct_from_external_api! {
    struct_name = "StructName",
    url_api = "https://api.example.com/data",

    // HTTP настройки:
    method = "GET",           // HTTP метод (default: "GET")
    timeout = 30000,          // Таймаут в мс (default: 30000)

    // Ограничения:
    max_depth = 5,            // Максимальная глубина вложенности
    max_entity_count = 100,   // Максимум элементов массива

    // Генерация:
    serde = true,
    default = false,
    optional = false,

    // Аутентификация (один из):
    auth_bearer = "token",
    auth_api_key = "Header-Name:value",
    auth_basic = "username:password",
}
```

### Примеры

**Публичный API:**

```rust
use unistructgen_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "GitHubUser",
    url_api = "https://api.github.com/users/octocat"
}

// GitHubUser сгенерирован со всеми полями из ответа
```

**API возвращающий массив:**

```rust
struct_from_external_api! {
    struct_name = "Todo",
    url_api = "https://jsonplaceholder.typicode.com/todos"
}

// Макрос автоматически извлекает первый элемент массива
// для определения структуры
```

**С аутентификацией:**

```rust
// Bearer Token
struct_from_external_api! {
    struct_name = "UserProfile",
    url_api = "https://api.myservice.com/me",
    auth_bearer = env!("API_TOKEN")
}

// API Key
struct_from_external_api! {
    struct_name = "WeatherData",
    url_api = "https://api.weather.com/current",
    auth_api_key = "X-API-Key:your_key"
}

// Basic Auth
struct_from_external_api! {
    struct_name = "PrivateData",
    url_api = "https://internal.api.com/data",
    auth_basic = "admin:secret"
}
```

---

## 📜 openapi_to_rust!

Генерирует типы и клиент из OpenAPI спецификации.

### Источники

**Из файла:**

```rust
openapi_to_rust! {
    file = "openapi.yaml"
}
```

**Из URL:**

```rust
openapi_to_rust! {
    url = "https://api.example.com/openapi.yaml",
    timeout = 30000,
    auth_bearer = env!("TOKEN")
}
```

**Inline спецификация:**

```rust
openapi_to_rust! {
    spec = r#"
openapi: 3.0.0
info:
  title: My API
  version: 1.0.0
components:
  schemas:
    User:
      type: object
      required:
        - id
        - email
      properties:
        id:
          type: integer
          format: int64
        email:
          type: string
          format: email
    "#
}
```

### Параметры

```rust
openapi_to_rust! {
    // Источник (один из):
    file = "spec.yaml",
    url = "https://...",
    spec = r#"..."#,

    // Генерация:
    generate_client = true,      // API клиент trait
    generate_validation = true,  // #[validate(...)]
    serde = true,
    default = false,

    // Для URL:
    timeout = 30000,
    auth_bearer = "token",
    auth_api_key = "Header:value",
    auth_basic = "user:pass",
}
```

### Пример

```rust
use unistructgen_macro::openapi_to_rust;

openapi_to_rust! {
    file = "petstore.yaml",
    generate_client = true,
    generate_validation = true
}

fn main() {
    let pet = Pet {
        id: Some(1),
        name: "Fluffy".to_string(),
        status: Some(PetStatus::Available),
    };

    // Валидация
    pet.validate().expect("Invalid pet");

    // Сериализация
    let json = serde_json::to_string(&pet).unwrap();
}
```

---

## 🔐 Аутентификация

### Bearer Token

```rust
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/protected",
    auth_bearer = "eyJhbGciOiJIUzI1NiIs..."
}
// Header: Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

### API Key

```rust
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/data",
    auth_api_key = "X-API-Key:sk_live_abc123"
}
// Header: X-API-Key: sk_live_abc123
```

### Basic Auth

```rust
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/secure",
    auth_basic = "admin:password123"
}
// Header: Authorization: Basic YWRtaW46cGFzc3dvcmQxMjM=
```

### Переменные окружения

```rust
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/data",
    auth_bearer = env!("MY_API_TOKEN")
}

// Перед компиляцией:
// export MY_API_TOKEN="your_secret_token"
```

---

## 🌐 Compile-time Fetch Controls

Для макросов, которые обращаются к сети во время компиляции (`struct_from_external_api!`, `openapi_to_rust!` с `url`, `env_file` по HTTP), доступны переменные окружения:

- `UNISTRUCTGEN_FETCH_OFFLINE=1` — запрет сети, только кеш
- `UNISTRUCTGEN_FETCH_CACHE=0` — отключить кеш
- `UNISTRUCTGEN_FETCH_CACHE_DIR=/path` — путь к кешу
- `UNISTRUCTGEN_FETCH_TIMEOUT_MS=60000` — таймаут (мс)

---

## 🔍 Type Inference

### Автоматическое определение типов

| JSON | Pattern | Rust Type |
|------|---------|-----------|
| Number (int) | - | `i64` / `u64` |
| Number (float) | - | `f64` |
| String | ISO 8601 datetime | `chrono::DateTime<Utc>` |
| String | UUID format | `uuid::Uuid` |
| String | other | `String` |
| Boolean | - | `bool` |
| Array | - | `Vec<T>` |
| Object | - | Nested struct |
| Null | - | `Option<serde_json::Value>` |

### Санитизация имён полей

| Оригинал | Результат |
|----------|-----------|
| `camelCase` | `camel_case` + `#[serde(rename = "camelCase")]` |
| `PascalCase` | `pascal_case` + rename |
| `kebab-case` | `kebab_case` + rename |
| `type` (keyword) | `type_` |

---

## 📝 Примеры

### E-commerce

```rust
generate_struct_from_json! {
    name = "Product",
    json = r#"{
        "id": "prod_123",
        "name": "Wireless Headphones",
        "price": 79.99,
        "in_stock": true,
        "categories": ["electronics", "audio"]
    }"#
}
```

### Configuration

```rust
#[json_struct(name = "AppConfig", default = true)]
const CONFIG: &str = r#"{
    "server": { "host": "0.0.0.0", "port": 8080 },
    "database": { "url": "postgres://localhost/db" }
}"#;
```

### External API

```rust
struct_from_external_api! {
    struct_name = "WeatherForecast",
    url_api = "https://api.weather.com/current?city=NYC",
    auth_api_key = "X-API-Key:your_key"
}
```

### OpenAPI Full

```rust
openapi_to_rust! {
    file = "api-spec.yaml",
    generate_client = true,
    generate_validation = true
}

async fn example() -> Result<(), ApiError> {
    let client = ApiClient::new("https://api.example.com");
    let users = client.list_users(Some(10)).await?;
    Ok(())
}
```

---

## ⚠️ Ограничения

1. **Compile-time network** — `struct_from_external_api!` делает HTTP-запросы во время компиляции
2. **Статические данные** — типы фиксированы на момент компиляции
3. **Секреты** — используйте `env!()` для переменных окружения

---

## 🔗 Связанные модули

- [unistructgen-core](../core/README.md) — IR, трейты, pipeline
- [unistructgen-codegen](../codegen/README.md) — Rust генератор
- [unistructgen-json-parser](../parsers/json_parser/README.md) — JSON парсер
- [unistructgen-openapi-parser](../parsers/openapi_parser/README.md) — OpenAPI парсер
- [unistructgen](../cli/README.md) — CLI инструмент

---

## 📜 Лицензия

MIT или Apache-2.0 — на ваш выбор.
