# 📄 UniStructGen JSON Parser

**Интеллектуальный парсер JSON с автоматическим выводом типов**

[![Crate](https://img.shields.io/crates/v/unistructgen-json-parser.svg)](https://crates.io/crates/unistructgen-json-parser)
[![Docs](https://docs.rs/unistructgen-json-parser/badge.svg)](https://docs.rs/unistructgen-json-parser)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../../LICENSE-MIT)

---

## 📋 Содержание

- [Обзор](#-обзор)
- [Установка](#-установка)
- [Быстрый старт](#-быстрый-старт)
- [Smart Type Inference](#-smart-type-inference)
- [ParserOptions](#-parseroptions)
- [JsonParserBuilder](#-jsonparserbuilder)
- [Кастомные детекторы](#-кастомные-детекторы)
- [Обработка вложенных объектов](#-обработка-вложенных-объектов)
- [Санитизация имён полей](#-санитизация-имён-полей)
- [Обработка ошибок](#-обработка-ошибок)
- [Примеры](#-примеры)

---

## 🎯 Обзор

`unistructgen-json-parser` преобразует JSON-данные в IR (Intermediate Representation) с интеллектуальным выводом типов:

### Ключевые возможности

| Возможность | Описание |
|-------------|----------|
| **Smart Type Inference** | Автоматическое определение DateTime, UUID, Email, URL |
| **Nested Objects** | Генерация отдельных структур для вложенных объектов |
| **Field Sanitization** | Преобразование camelCase/kebab-case в snake_case |
| **Serde Integration** | Автоматические #[serde(rename = "...")] атрибуты |
| **Extensible Detectors** | Добавление кастомных детекторов типов |
| **Array Handling** | Корректный вывод типов элементов массивов |

---

## 📦 Установка

```toml
[dependencies]
unistructgen-json-parser = "0.1"
unistructgen-core = "0.1"
```

---

## 🚀 Быстрый старт

```rust
use unistructgen_json_parser::{JsonParser, ParserOptions};
use unistructgen_core::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{
        "id": 12345,
        "name": "Alice Smith",
        "email": "alice@example.com",
        "created_at": "2024-01-15T10:30:00Z",
        "is_active": true,
        "balance": 1250.50
    }"#;

    let mut parser = JsonParser::new(ParserOptions {
        struct_name: "User".to_string(),
        derive_serde: true,
        ..Default::default()
    });

    let module = parser.parse(json)?;

    // module.types содержит IRStruct "User" с полями:
    // - id: i64
    // - name: String
    // - email: String
    // - created_at: DateTime  ← Автоматически определено!
    // - is_active: bool
    // - balance: f64

    Ok(())
}
```

---

## 🧠 Smart Type Inference

Парсер автоматически определяет специальные типы из строковых значений:

### DateTime Detection

```rust
// Все эти строки распознаются как DateTime:
"2024-01-15T10:30:00Z"           // ISO 8601 с Z
"2024-01-15T10:30:00+00:00"      // ISO 8601 с timezone
"2024-01-15T10:30:00.123Z"       // С миллисекундами
"2024-01-15T00:00:00"            // Локальное время
```

**Результат:** `IRTypeRef::Primitive(PrimitiveKind::DateTime)`

### UUID Detection

```rust
// Стандартный формат UUID (8-4-4-4-12):
"550e8400-e29b-41d4-a716-446655440000"
"a1b2c3d4-e5f6-7890-abcd-ef1234567890"
```

**Результат:** `IRTypeRef::Primitive(PrimitiveKind::Uuid)`

### Numeric Types

```rust
// Целые числа
42        → i64
-100      → i64
9007199254740991  → i64

// Большие целые числа
18446744073709551615  → u64 (если положительное)

// Числа с плавающей точкой
3.14      → f64
-0.001    → f64
```

### Boolean

```rust
true      → bool
false     → bool
```

### Null

```rust
null      → Option<serde_json::Value>
```

---

## ⚙️ ParserOptions

Конфигурация парсера.

### Все опции

| Опция | Тип | По умолчанию | Описание |
|-------|-----|--------------|----------|
| `struct_name` | `String` | `"Root"` | Имя корневой структуры |
| `derive_serde` | `bool` | `true` | Добавлять serde derives |
| `derive_default` | `bool` | `false` | Добавлять Default derive |
| `make_fields_optional` | `bool` | `false` | Все поля как Option<T> |

### Примеры конфигурации

```rust
use unistructgen_json_parser::ParserOptions;

// Минимальная конфигурация
let options = ParserOptions {
    struct_name: "Config".to_string(),
    ..Default::default()
};

// Для API-ответов (все поля опциональные)
let api_options = ParserOptions {
    struct_name: "ApiResponse".to_string(),
    derive_serde: true,
    derive_default: true,
    make_fields_optional: true,
};

// Без serde
let plain_options = ParserOptions {
    struct_name: "Data".to_string(),
    derive_serde: false,
    derive_default: false,
    make_fields_optional: false,
};
```

---

## 🏗️ JsonParserBuilder

Fluent-билдер для создания парсера.

```rust
use unistructgen_json_parser::JsonParserBuilder;

let parser = JsonParserBuilder::new()
    .struct_name("UserProfile")
    .derive_serde(true)
    .derive_default(true)
    .make_fields_optional(false)
    .build();
```

---

## 🔍 Кастомные детекторы

Расширяйте систему вывода типов своими детекторами.

### CustomTypeDetector trait

```rust
use unistructgen_json_parser::inference::{CustomTypeDetector, InferenceError};
use unistructgen_core::IRTypeRef;

pub trait CustomTypeDetector: Send + Sync {
    /// Имя детектора (для логов и ошибок)
    fn name(&self) -> &str;

    /// Попытка определить тип
    /// - Ok(Some(type)) — тип определён
    /// - Ok(None) — этот детектор не подходит
    /// - Err(...) — ошибка детекции
    fn detect(&self, value: &str) -> Result<Option<IRTypeRef>, InferenceError>;

    /// Описание (опционально)
    fn description(&self) -> Option<&str> { None }
}
```

### Встроенные детекторы

| Детектор | Что определяет | Результат |
|----------|----------------|-----------|
| `DateTimeDetector` | ISO 8601 datetime | `PrimitiveKind::DateTime` |
| `UuidDetector` | UUID (8-4-4-4-12) | `PrimitiveKind::Uuid` |
| `EmailDetector` | Email адреса | `Named("Email")` |
| `UrlDetector` | HTTP/HTTPS URLs | `Named("Url")` |

### Пример кастомного детектора

```rust
use unistructgen_json_parser::inference::{CustomTypeDetector, InferenceError};
use unistructgen_core::{IRTypeRef, PrimitiveKind};

/// Детектор для номеров телефонов
struct PhoneDetector;

impl CustomTypeDetector for PhoneDetector {
    fn name(&self) -> &str { "PhoneDetector" }

    fn description(&self) -> Option<&str> {
        Some("Detects phone numbers in E.164 format")
    }

    fn detect(&self, value: &str) -> Result<Option<IRTypeRef>, InferenceError> {
        // Простая эвристика: начинается с + и содержит 10-15 цифр
        if value.starts_with('+') && value.len() >= 10 && value.len() <= 16 {
            let digits_only = value[1..].chars().all(|c| c.is_ascii_digit());
            if digits_only {
                return Ok(Some(IRTypeRef::Named("PhoneNumber".to_string())));
            }
        }
        Ok(None)
    }
}
```

### SmartTypeInference

Комбинация нескольких детекторов:

```rust
use unistructgen_json_parser::inference::{
    SmartTypeInference, DateTimeDetector, UuidDetector,
    EmailDetector, UrlDetector, TypeInferenceStrategy
};
use serde_json::json;

// Стандартная конфигурация
let inference = SmartTypeInference::new();

// С расширенными детекторами
let inference = SmartTypeInference::with_common_detectors();
// Включает: DateTimeDetector, UuidDetector, EmailDetector, UrlDetector

// С кастомными детекторами
let inference = SmartTypeInference::new()
    .add_detector(Box::new(PhoneDetector))
    .add_detector(Box::new(CurrencyDetector));

// Использование
let value = json!("user@example.com");
let ty = inference.infer(&value, "email")?;
// → IRTypeRef::Named("Email")
```

---

## 📦 Обработка вложенных объектов

Парсер автоматически создаёт отдельные структуры для вложенных объектов.

### Пример

```rust
let json = r#"{
    "user": {
        "id": 1,
        "profile": {
            "bio": "Developer",
            "location": "NYC"
        }
    },
    "settings": {
        "theme": "dark",
        "notifications": true
    }
}"#;

let mut parser = JsonParser::new(ParserOptions {
    struct_name: "AppData".to_string(),
    ..Default::default()
});

let module = parser.parse(json)?;
// Генерирует 4 структуры:
// 1. Profile { bio: String, location: String }
// 2. User { id: i64, profile: Profile }
// 3. Settings { theme: String, notifications: bool }
// 4. AppData { user: User, settings: Settings }
```

### Уникальные имена

Парсер автоматически разрешает конфликты имён:

```rust
let json = r#"{
    "user": { "data": { "id": 1 } },
    "order": { "data": { "id": 2, "total": 100 } }
}"#;

// Генерирует:
// - Data  (для user.data)
// - Data1 (для order.data, т.к. Data уже занято)
```

---

## 🔠 Санитизация имён полей

Парсер преобразует имена полей в валидные Rust идентификаторы.

### Правила преобразования

| Оригинал | Результат | Правило |
|----------|-----------|---------|
| `camelCase` | `camel_case` | CamelCase → snake_case |
| `PascalCase` | `pascal_case` | PascalCase → snake_case |
| `kebab-case` | `kebab_case` | Дефис → подчёркивание |
| `space case` | `space_case` | Пробел → подчёркивание |
| `123field` | `_123field` | Цифра в начале → добавить _ |
| `type` | `type_` | Rust keywords → добавить _ |

### Сохранение оригинального имени

Если имя изменено, парсер сохраняет оригинал и добавляет serde атрибут:

```rust
// JSON: { "userName": "Alice" }

// Сгенерированный код:
#[serde(rename = "userName")]
pub user_name: String,
```

### Rust Keywords

Следующие слова экранируются добавлением `_`:

```
as, break, const, continue, crate, else, enum, extern,
false, fn, for, if, impl, in, let, loop, match, mod,
move, mut, pub, ref, return, self, Self, static, struct,
super, trait, true, type, unsafe, use, where, while,
async, await, dyn
```

---

## ❌ Обработка ошибок

### JsonParserError

```rust
use unistructgen_json_parser::JsonParserError;

pub enum JsonParserError {
    /// Синтаксическая ошибка JSON
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
        source: serde_json::Error,
    },

    /// Неверная структура JSON
    InvalidStructure {
        path: String,      // "$.user.address"
        expected: String,  // "object"
        found: String,     // "array"
    },

    /// Не удалось определить тип
    TypeInferenceFailed {
        field: String,
        path: String,
        reason: String,
        suggestion: Option<String>,
    },

    /// Конфликт типов (при мердже семплов)
    TypeConflict {
        field: String,
        path: String,
        type1: String,
        type2: String,
    },

    /// Невалидное имя поля
    InvalidFieldName {
        original: String,
        path: String,
        reason: String,
    },

    /// Превышена максимальная глубина вложенности
    MaxDepthExceeded {
        path: String,
        max_depth: usize,
    },
}
```

### Примеры обработки

```rust
use unistructgen_json_parser::{JsonParser, ParserOptions, JsonParserError};
use unistructgen_core::Parser;

let mut parser = JsonParser::new(ParserOptions::default());

match parser.parse(json) {
    Ok(module) => {
        println!("Parsed {} types", module.types.len());
    }
    Err(JsonParserError::SyntaxError { line, column, message, .. }) => {
        eprintln!("JSON syntax error at {}:{}: {}", line, column, message);
    }
    Err(JsonParserError::InvalidStructure { path, expected, found }) => {
        eprintln!("At {}: expected {}, found {}", path, expected, found);
    }
    Err(e) => {
        eprintln!("Parse error: {}", e);
    }
}
```

### Валидация без полного парсинга

```rust
// Быстрая проверка валидности JSON
if parser.validate(json).is_ok() {
    // JSON валиден, можно парсить
}
```

---

## 📝 Примеры

### API Response

```rust
let api_response = r#"{
    "success": true,
    "data": {
        "users": [
            {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "name": "Alice",
                "email": "alice@example.com",
                "created_at": "2024-01-15T10:30:00Z",
                "roles": ["admin", "user"]
            }
        ],
        "total": 100,
        "page": 1
    },
    "error": null
}"#;

let mut parser = JsonParser::new(ParserOptions {
    struct_name: "ApiResponse".to_string(),
    derive_serde: true,
    derive_default: true,
    ..Default::default()
});

let module = parser.parse(api_response)?;

// Генерирует структуры:
// - UsersItem { id: Uuid, name: String, email: String, created_at: DateTime, roles: Vec<String> }
// - Data { users: Vec<UsersItem>, total: i64, page: i64 }
// - ApiResponse { success: bool, data: Data, error: Option<serde_json::Value> }
```

### Configuration File

```rust
let config = r#"{
    "server": {
        "host": "0.0.0.0",
        "port": 8080,
        "timeout_ms": 30000
    },
    "database": {
        "url": "postgres://localhost/db",
        "pool_size": 10,
        "ssl": true
    },
    "features": {
        "enable_cache": true,
        "cache_ttl": 3600
    }
}"#;

let mut parser = JsonParser::new(ParserOptions {
    struct_name: "AppConfig".to_string(),
    ..Default::default()
});

let module = parser.parse(config)?;
```

### E-commerce Order

```rust
let order = r#"{
    "order_id": "ORD-2024-001",
    "customer": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "email": "customer@example.com"
    },
    "items": [
        {
            "product_id": "PROD-001",
            "name": "Widget",
            "quantity": 2,
            "unit_price": 29.99
        }
    ],
    "total": 59.98,
    "currency": "USD",
    "created_at": "2024-01-15T14:30:00Z",
    "shipped_at": null
}"#;

let mut parser = JsonParser::new(ParserOptions {
    struct_name: "Order".to_string(),
    derive_serde: true,
    ..Default::default()
});

let module = parser.parse(order)?;

// Генерирует:
// - Customer { id: Uuid, email: String }
// - ItemsItem { product_id: String, name: String, quantity: i64, unit_price: f64 }
// - Order { order_id: String, customer: Customer, items: Vec<ItemsItem>,
//           total: f64, currency: String, created_at: DateTime, shipped_at: Option<Value> }
```

---

## 🔗 Связанные модули

- [unistructgen-core](../../core/README.md) — IR, трейты, pipeline
- [unistructgen-codegen](../../codegen/README.md) — Rust генератор
- [unistructgen-markdown-parser](../markdown_parser/README.md) — Markdown парсер
- [unistructgen-openapi-parser](../openapi_parser/README.md) — OpenAPI парсер

---

## 📜 Лицензия

MIT или Apache-2.0 — на ваш выбор.
