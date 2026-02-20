# 💻 UniStructGen CLI

**Мощный инструмент командной строки для генерации Rust-кода**

[![Crate](https://img.shields.io/crates/v/unistructgen.svg)](https://crates.io/crates/unistructgen)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../LICENSE-MIT)

---

## 📋 Содержание

- [Обзор](#-обзор)
- [Установка](#-установка)
- [Команды](#-команды)
- [Команда generate](#-команда-generate)
- [Команда client](#-команда-client)
- [Примеры](#-примеры)

---

## 🎯 Обзор

`unistructgen` CLI — это инструмент командной строки для генерации типобезопасного Rust-кода из различных источников данных:

| Источник | Команда | Результат |
|----------|---------|-----------|
| JSON | `generate` | Rust structs |
| Markdown | `generate` | Rust structs |
| SQL | `generate` | Rust structs |
| OpenAPI | `client` | API client scaffold |

---

## 📦 Установка

### Из crates.io

```bash
cargo install unistructgen
```

### Из исходников

```bash
git clone https://github.com/your-org/unistructgen.git
cd unistructgen
cargo install --path cli
```

### Проверка установки

```bash
unistructgen --version
# unistructgen 0.1.0

unistructgen --help
```

---

## 📖 Команды

```
unistructgen - Generate Rust structs from JSON, Markdown, or SQL

USAGE:
    unistructgen <COMMAND>

COMMANDS:
    generate    Generate Rust code from input
    client      Generate a typed HTTP client scaffold from OpenAPI specification
    help        Print this message or the help of the given subcommand(s)

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
```

---

## 🔧 Команда generate

Генерирует Rust-структуры из JSON, Markdown или SQL файлов.

### Синтаксис

```bash
unistructgen generate [OPTIONS] --input <FILE>
```

### Опции

| Опция | Короткая | Тип | По умолчанию | Описание |
|-------|----------|-----|--------------|----------|
| `--input` | `-i` | PATH | **обязательно** | Входной файл |
| `--output` | `-o` | PATH | stdout | Выходной файл |
| `--format` | `-f` | STRING | auto | Формат входа (json, markdown, sql) |
| `--name` | `-n` | STRING | "Root" | Имя корневой структуры |
| `--serde` | | BOOL | true | Добавить serde derives |
| `--default` | | BOOL | false | Добавить Default derive |
| `--optional` | | BOOL | false | Все поля как Option<T> |

### Примеры

**Базовое использование:**

```bash
# Из JSON (автоопределение формата по расширению)
unistructgen generate --input user.json --name User

# С указанием выходного файла
unistructgen generate -i data.json -o src/models/user.rs -n User

# Из Markdown
unistructgen generate -i API_SPEC.md -o models.rs -n ApiModel
```

**С опциями:**

```bash
# Без serde derives
unistructgen generate -i data.json -n Config --serde false

# С Default derive
unistructgen generate -i data.json -n Config --default true

# Все поля опциональные
unistructgen generate -i api_response.json -n ApiResponse --optional true

# Комбинация
unistructgen generate -i config.json -o config.rs -n AppConfig \
    --serde true --default true
```

**Явное указание формата:**

```bash
# Если расширение не стандартное
unistructgen generate -i data.txt -f json -n Data
unistructgen generate -i schema.txt -f markdown -n Schema
```

---

## 🔌 Команда client

Генерирует типизированный HTTP-клиент scaffold из OpenAPI спецификации.

### Синтаксис

```bash
unistructgen client [OPTIONS]
```

### Опции

| Опция | Короткая | Тип | По умолчанию | Описание |
|-------|----------|-----|--------------|----------|
| `--spec` | `-s` | PATH | - | Путь к OpenAPI файлу |
| `--url` | `-u` | URL | - | URL для загрузки спецификации |
| `--output` | `-o` | PATH | ./generated-client | Директория для вывода |
| `--name` | `-n` | STRING | "Api" | Имя клиента |
| `--examples` | | BOOL | true | Генерировать примеры (`true`/`false`) |

### Примеры

**Из локального файла:**

```bash
unistructgen client --spec openapi.yaml --output ./src/api --name GitHub
```

**Из URL:**

```bash
unistructgen client --url https://api.example.com/openapi.yaml --name ExampleApi
```

**Полная настройка:**

```bash
unistructgen client \
    --spec petstore.yaml \
    --output ./src/generated \
    --name Petstore \
    --examples false
```

### Структура вывода

```
./src/generated/
├── client.rs       # Клиентский код (методы по paths + auth)
├── types.rs        # Сгенерированные структуры
├── lib.rs          # Реэкспорты и публичный API
├── Cargo.toml      # Зависимости клиента
├── README.md       # Документация клиента
└── examples/       # Примеры использования (если --examples true)
    └── basic.rs
```

---

## 📝 Примеры

### Пример 1: API Response

**Входной файл `response.json`:**

```json
{
    "success": true,
    "data": {
        "user": {
            "id": 12345,
            "email": "user@example.com",
            "created_at": "2024-01-15T10:30:00Z"
        }
    },
    "error": null
}
```

**Команда:**

```bash
unistructgen generate -i response.json -o src/models.rs -n ApiResponse
```

**Результат (`src/models.rs`):**

```rust
// Generated by unistructgen v0.1.0
// Do not edit this file manually

#![allow(dead_code)]
#![allow(unused_imports)]

#[derive(serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Data {
    pub user: User,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub data: Data,
    pub error: Option<serde_json::Value>,
}
```

### Пример 2: Configuration

**Входной файл `config.json`:**

```json
{
    "server": {
        "host": "0.0.0.0",
        "port": 8080
    },
    "database": {
        "url": "postgres://localhost/db",
        "pool_size": 10
    }
}
```

**Команда:**

```bash
unistructgen generate -i config.json -n AppConfig --default true
```

**Результат:**

```rust
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Server {
    pub host: String,
    pub port: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Database {
    pub url: String,
    pub pool_size: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AppConfig {
    pub server: Server,
    pub database: Database,
}
```

### Пример 3: Markdown Documentation

**Входной файл `schema.md`:**

```markdown
# User Schema

| Field     | Type     | Description       | Required |
|-----------|----------|-------------------|----------|
| id        | uuid     | User identifier   | Yes      |
| username  | string   | Login name        | Yes      |
| email     | string   | Email address     | Yes      |
| role      | string   | User role         | No       |
```

**Команда:**

```bash
unistructgen generate -i schema.md -o user.rs -n User
```

**Результат:**

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct User {
    /// User identifier
    pub id: uuid::Uuid,
    /// Login name
    pub username: String,
    /// Email address
    pub email: String,
    /// User role
    pub role: Option<String>,
}
```

### Пример 4: OpenAPI Client

**Команда:**

```bash
unistructgen client \
    --url https://petstore3.swagger.io/api/v3/openapi.json \
    --output ./src/petstore \
    --name Petstore
```

**Использование сгенерированного клиента:**

```rust
use petstore::{PetstoreClient, Pet, PetStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PetstoreClient::new("https://petstore3.swagger.io/api/v3");

    // Получить питомца по ID
    let pet = client.get_pet_by_id(1).await?;
    println!("Pet: {:?}", pet);

    // Создать нового питомца
    let new_pet = Pet {
        id: None,
        name: "Fluffy".to_string(),
        status: Some(PetStatus::Available),
        ..Default::default()
    };
    let created = client.add_pet(new_pet).await?;
    println!("Created: {:?}", created);

    Ok(())
}
```

---

## 🔄 Интеграция в CI/CD

### GitHub Actions

```yaml
name: Generate Models

on:
  push:
    paths:
      - 'schemas/**'

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install unistructgen
        run: cargo install unistructgen

      - name: Generate from JSON schemas
        run: |
          for file in schemas/*.json; do
            name=$(basename "$file" .json)
            unistructgen generate -i "$file" -o "src/models/${name}.rs" -n "${name^}"
          done

      - name: Commit changes
        uses: stefanzweifel/git-auto-commit-action@v5
        with:
          commit_message: "chore: regenerate models"
```

### Makefile

```makefile
.PHONY: generate-models generate-client

generate-models:
	unistructgen generate -i schemas/user.json -o src/models/user.rs -n User
	unistructgen generate -i schemas/order.json -o src/models/order.rs -n Order

generate-client:
	unistructgen client --spec api/openapi.yaml --output src/api --name MyApi

generate: generate-models generate-client
```

---

## 🔗 Связанные модули

- [unistructgen-core](../core/README.md) — IR, трейты, pipeline
- [unistructgen-codegen](../codegen/README.md) — Rust генератор
- [unistructgen-json-parser](../parsers/json_parser/README.md) — JSON парсер
- [unistructgen-markdown-parser](../parsers/markdown_parser/README.md) — Markdown парсер
- [unistructgen-openapi-parser](../parsers/openapi_parser/README.md) — OpenAPI парсер
- [unistructgen-macro](../proc-macro/README.md) — Процедурные макросы

---

## 📜 Лицензия

MIT или Apache-2.0 — на ваш выбор.
