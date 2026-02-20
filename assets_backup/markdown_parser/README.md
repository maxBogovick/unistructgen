# 📝 UniStructGen Markdown Parser

**Генерация Rust-структур из Markdown-таблиц документации**

[![Crate](https://img.shields.io/crates/v/unistructgen-markdown-parser.svg)](https://crates.io/crates/unistructgen-markdown-parser)
[![Docs](https://docs.rs/unistructgen-markdown-parser/badge.svg)](https://docs.rs/unistructgen-markdown-parser)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../../LICENSE-MIT)

---

## 📋 Содержание

- [Обзор](#-обзор)
- [Установка](#-установка)
- [Быстрый старт](#-быстрый-старт)
- [Формат таблиц](#-формат-таблиц)
- [ParserOptions](#-parseroptions)
- [Вывод типов](#-вывод-типов)
- [Обработка ошибок](#-обработка-ошибок)
- [Примеры](#-примеры)

---

## 🎯 Обзор

`unistructgen-markdown-parser` преобразует Markdown-таблицы в IR (Intermediate Representation). Это позволяет:

- **Documentation-first** — писать документацию, получать код
- **Single Source of Truth** — таблицы в README = модели в коде
- **Collaboration** — не-программисты могут определять структуры данных

### Идеальный сценарий

```
📄 API_SPEC.md (документация)
        ↓
  [MarkdownParser]
        ↓
   IR Module
        ↓
  [RustRenderer]
        ↓
📄 models.rs (код)
```

---

## 📦 Установка

```toml
[dependencies]
unistructgen-markdown-parser = "0.1"
unistructgen-core = "0.1"
```

---

## 🚀 Быстрый старт

```rust
use unistructgen_markdown_parser::{MarkdownParser, MarkdownParserOptions};
use unistructgen_core::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let markdown = r#"
# User Model

This table describes the User entity:

| Field Name | Type     | Description          | Required |
|------------|----------|----------------------|----------|
| id         | uuid     | Unique identifier    | Yes      |
| username   | string   | User's login name    | Yes      |
| email      | string   | Contact email        | Yes      |
| age        | integer  | User's age           | No       |
| created_at | datetime | Registration date    | Yes      |
"#;

    let mut parser = MarkdownParser::new(MarkdownParserOptions {
        struct_name: "User".to_string(),
        derive_serde: true,
        ..Default::default()
    });

    let module = parser.parse(markdown)?;

    // module.types содержит IRStruct "User" с полями:
    // - id: Uuid
    // - username: String
    // - email: String
    // - age: Option<i32>  ← Автоматически Option, т.к. Required = No
    // - created_at: DateTime

    Ok(())
}
```

---

## 📊 Формат таблиц

### Обязательные колонки

Парсер ищет колонки по ключевым словам:

| Колонка | Ключевые слова | Обязательно |
|---------|----------------|-------------|
| **Name** | `name`, `field`, `property` | ✅ Да |
| **Type** | `type` | ✅ Да |
| **Description** | `desc`, `description`, `comment` | ❌ Нет |
| **Required** | `required`, `optional` | ❌ Нет |

### Примеры валидных таблиц

**Минимальная таблица:**

```markdown
| Field | Type   |
|-------|--------|
| id    | i64    |
| name  | string |
```

**Полная таблица:**

```markdown
| Property Name | Data Type | Description        | Required |
|---------------|-----------|--------------------|---------:|
| user_id       | uuid      | Unique identifier  | Yes      |
| full_name     | string    | User's full name   | Yes      |
| bio           | text      | Biography          | No       |
```

**Вариации заголовков:**

```markdown
| field_name | type    | comment            | optional |
|------------|---------|--------------------|---------:|
| id         | integer | Primary key        | false    |
| status     | string  | Current status     | true     |
```

---

## ⚙️ ParserOptions

| Опция | Тип | По умолчанию | Описание |
|-------|-----|--------------|----------|
| `struct_name` | `String` | `"Root"` | Имя генерируемой структуры |
| `derive_serde` | `bool` | `true` | Добавлять serde derives |
| `derive_default` | `bool` | `false` | Добавлять Default derive |
| `make_fields_optional` | `bool` | `false` | Все поля как Option<T> |

### Примеры

```rust
use unistructgen_markdown_parser::MarkdownParserOptions;

// Для API-моделей
let api_options = MarkdownParserOptions {
    struct_name: "ApiRequest".to_string(),
    derive_serde: true,
    derive_default: true,
    make_fields_optional: false,
};

// Для конфигурации (все опционально)
let config_options = MarkdownParserOptions {
    struct_name: "Config".to_string(),
    derive_serde: true,
    derive_default: true,
    make_fields_optional: true,
};
```

---

## 🔤 Вывод типов

### Поддерживаемые типы

| Markdown | Rust | Примечание |
|----------|------|------------|
| `string`, `text`, `varchar`, `str` | `String` | Строковые типы |
| `integer`, `int`, `i32`, `number` | `i32` | 32-bit целые |
| `long`, `i64`, `bigint` | `i64` | 64-bit целые |
| `float`, `double`, `f64`, `decimal` | `f64` | Числа с плавающей точкой |
| `boolean`, `bool` | `bool` | Булевы значения |
| `date`, `datetime`, `timestamp` | `DateTime` | Дата и время |
| `uuid`, `guid` | `Uuid` | UUID |
| `json`, `object`, `map` | `serde_json::Value` | JSON значения |
| `vec<T>`, `array<T>`, `T[]` | `Vec<T>` | Массивы |
| *Любое другое* | `Named(PascalCase)` | Ссылка на тип |

### Примеры типов

```markdown
| Field      | Type           | Description              |
|------------|----------------|--------------------------|
| id         | uuid           | → Uuid                   |
| count      | integer        | → i32                    |
| amount     | decimal        | → f64                    |
| active     | boolean        | → bool                   |
| created    | datetime       | → DateTime               |
| tags       | vec<string>    | → Vec<String>            |
| items      | string[]       | → Vec<String>            |
| metadata   | json           | → serde_json::Value      |
| address    | Address        | → Address (named type)   |
```

---

## ❌ Обработка ошибок

### MarkdownParserError

```rust
use unistructgen_markdown_parser::MarkdownParserError;

pub enum MarkdownParserError {
    /// В документе нет таблиц
    NoTablesFound,

    /// Ошибка маппинга таблицы
    MappingError(String),

    /// Неверная структура колонок
    InvalidColumns(String),
}
```

### Обработка

```rust
use unistructgen_markdown_parser::{MarkdownParser, MarkdownParserOptions, MarkdownParserError};
use unistructgen_core::Parser;

let mut parser = MarkdownParser::new(MarkdownParserOptions::default());

match parser.parse(markdown) {
    Ok(module) => {
        println!("Parsed {} types", module.types.len());
    }
    Err(MarkdownParserError::NoTablesFound) => {
        eprintln!("No tables found in the markdown document");
    }
    Err(MarkdownParserError::InvalidColumns(msg)) => {
        eprintln!("Invalid table structure: {}", msg);
    }
    Err(e) => {
        eprintln!("Parse error: {}", e);
    }
}
```

---

## 📝 Примеры

### Database Schema Documentation

```markdown
# Database Schema

## Users Table

| Column Name  | Data Type    | Description                | Nullable |
|--------------|--------------|----------------------------|----------|
| id           | uuid         | Primary key                | No       |
| email        | varchar(255) | User's email (unique)      | No       |
| password_hash| varchar(255) | Bcrypt hash                | No       |
| first_name   | varchar(100) | First name                 | Yes      |
| last_name    | varchar(100) | Last name                  | Yes      |
| created_at   | timestamp    | Registration timestamp     | No       |
| updated_at   | timestamp    | Last update timestamp      | Yes      |
| deleted_at   | timestamp    | Soft delete timestamp      | Yes      |

## Orders Table

| Column Name  | Data Type | Description            | Nullable |
|--------------|-----------|------------------------|----------|
| id           | uuid      | Order ID               | No       |
| user_id      | uuid      | Reference to users.id  | No       |
| total_amount | decimal   | Order total            | No       |
| status       | string    | Order status           | No       |
| created_at   | timestamp | Order creation time    | No       |
```

```rust
// Парсинг первой таблицы
let mut parser = MarkdownParser::new(MarkdownParserOptions {
    struct_name: "User".to_string(),
    derive_serde: true,
    ..Default::default()
});

let module = parser.parse(markdown)?;
```

### API Documentation

```markdown
# Create User Endpoint

**POST** `/api/v1/users`

## Request Body

| Field    | Type   | Description           | Required |
|----------|--------|-----------------------|----------|
| email    | string | User's email address  | Yes      |
| password | string | Password (min 8 chars)| Yes      |
| name     | string | Display name          | No       |

## Response

| Field      | Type     | Description              |
|------------|----------|--------------------------|
| id         | uuid     | Created user ID          |
| email      | string   | User's email             |
| name       | string   | Display name (or null)   |
| created_at | datetime | Creation timestamp       |
```

### Configuration Documentation

```markdown
# Application Configuration

| Parameter       | Type    | Default  | Description                    |
|-----------------|---------|----------|--------------------------------|
| server_host     | string  | 0.0.0.0  | Server bind address            |
| server_port     | integer | 8080     | Server port                    |
| database_url    | string  | -        | PostgreSQL connection string   |
| redis_url       | string  | -        | Redis connection (optional)    |
| log_level       | string  | info     | Logging level                  |
| enable_metrics  | boolean | true     | Enable Prometheus metrics      |
| max_connections | integer | 100      | Max database connections       |
```

### Event Schema

```markdown
# Event Types

## UserCreated Event

Emitted when a new user is registered.

| Field     | Type     | Description              |
|-----------|----------|--------------------------|
| event_id  | uuid     | Unique event identifier  |
| timestamp | datetime | When the event occurred  |
| user_id   | uuid     | Created user's ID        |
| email     | string   | User's email address     |
| source    | string   | Service that created it  |

## OrderPlaced Event

Emitted when an order is placed.

| Field       | Type      | Description            |
|-------------|-----------|------------------------|
| event_id    | uuid      | Event ID               |
| timestamp   | datetime  | Event timestamp        |
| order_id    | uuid      | Order ID               |
| user_id     | uuid      | Customer ID            |
| items       | json      | Order items array      |
| total       | decimal   | Order total            |
```

---

## 🔗 Связанные модули

- [unistructgen-core](../../core/README.md) — IR, трейты, pipeline
- [unistructgen-codegen](../../codegen/README.md) — Rust генератор
- [unistructgen-json-parser](../json_parser/README.md) — JSON парсер
- [unistructgen-openapi-parser](../openapi_parser/README.md) — OpenAPI парсер

---

## 🗺 Roadmap

- [ ] Поддержка множественных таблиц → множество структур
- [ ] Извлечение имени структуры из заголовка
- [ ] Поддержка вложенных типов через ссылки
- [ ] Поддержка enum из ограниченного набора значений
- [ ] Поддержка HTML-таблиц

---

## 📜 Лицензия

MIT или Apache-2.0 — на ваш выбор.
