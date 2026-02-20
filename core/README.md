# 🧠 UniStructGen Core

**Ядро библиотеки UniStructGen — фундамент для генерации типобезопасного кода**

[![Crate](https://img.shields.io/crates/v/unistructgen-core.svg)](https://crates.io/crates/unistructgen-core)
[![Docs](https://docs.rs/unistructgen-core/badge.svg)](https://docs.rs/unistructgen-core)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../LICENSE-MIT)

---

## 📋 Содержание

- [Обзор](#-обзор)
- [Установка](#-установка)
- [Intermediate Representation (IR)](#-intermediate-representation-ir)
- [Unified API](#-unified-api)
- [Трейты](#-трейты)
- [Трансформеры](#-трансформеры)
- [Pipeline](#-pipeline)
- [Plugin System](#-plugin-system)
- [Visitor Pattern](#-visitor-pattern)
- [Примеры](#-примеры)

---

## 🎯 Обзор

`unistructgen-core` — это сердце экосистемы UniStructGen. Он предоставляет:

- **IR (Intermediate Representation)** — язык-агностичное представление типов
- **Трейты** — интерфейсы для парсеров, генераторов, трансформеров
- **Pipeline** — композируемый конвейер обработки
- **Plugin System** — расширяемая архитектура плагинов
- **Unified API** — эргономичный Builder API для генерации кода

### Принцип работы

```
Input (JSON/MD/OpenAPI)
        ↓
   [Parser] ─────────────→ IRModule
        ↓                      │
   [Plugins: after_parse]      │
        ↓                      │
   [Transformers] ←────────────┘
        ↓
   [CodeGenerator]
        ↓
   [Plugins: after_generate]
        ↓
   Output (Rust/JSON Schema via codegen)
```

---

## 📦 Установка

```toml
[dependencies]
unistructgen-core = "0.1"
```

---

## 🔷 Intermediate Representation (IR)

IR — это язык-независимое представление структур данных. Все парсеры генерируют IR, все генераторы потребляют IR.

### IRModule

Контейнер верхнего уровня для всех типов:

```rust
use unistructgen_core::{IRModule, IRType, IRStruct};

let mut module = IRModule::new("users".to_string());
module.add_type(IRType::Struct(IRStruct::new("User".to_string())));

// Сериализуем в JSON (для отладки или хранения)
let json = serde_json::to_string_pretty(&module)?;
```

### IRStruct

Определение структуры:

```rust
use unistructgen_core::{IRStruct, IRField, IRTypeRef, PrimitiveKind};

let mut user = IRStruct::new("User".to_string());
user.doc = Some("Пользователь системы".to_string());
user.add_derive("Default".to_string());

user.add_field(IRField::new(
    "id".to_string(),
    IRTypeRef::Primitive(PrimitiveKind::I64)
));

user.add_field(IRField::new(
    "email".to_string(),
    IRTypeRef::Primitive(PrimitiveKind::String)
));
```

### IREnum

Определение перечисления:

```rust
use unistructgen_core::{IREnum, IREnumVariant};

let mut status = IREnum {
    name: "UserStatus".to_string(),
    variants: vec![
        IREnumVariant {
            name: "Active".to_string(),
            source_value: None,
            doc: Some("Активный пользователь".to_string()),
        },
        IREnumVariant {
            name: "Suspended".to_string(),
            source_value: Some("suspended".to_string()), // serde rename
            doc: None,
        },
    ],
    derives: vec!["Debug".to_string(), "Clone".to_string()],
    doc: Some("Статус пользователя".to_string()),
};
```

### IRTypeRef

Ссылка на тип:

```rust
use unistructgen_core::{IRTypeRef, PrimitiveKind};

// Примитивы
let string_type = IRTypeRef::Primitive(PrimitiveKind::String);
let i64_type = IRTypeRef::Primitive(PrimitiveKind::I64);
let datetime = IRTypeRef::Primitive(PrimitiveKind::DateTime);
let uuid = IRTypeRef::Primitive(PrimitiveKind::Uuid);

// Option<T>
let optional_string = IRTypeRef::Option(Box::new(
    IRTypeRef::Primitive(PrimitiveKind::String)
));

// Vec<T>
let string_vec = IRTypeRef::Vec(Box::new(
    IRTypeRef::Primitive(PrimitiveKind::String)
));

// HashMap<K, V>
let string_map = IRTypeRef::Map(
    Box::new(IRTypeRef::Primitive(PrimitiveKind::String)),
    Box::new(IRTypeRef::Primitive(PrimitiveKind::I64)),
);

// Ссылка на другой тип
let address_ref = IRTypeRef::Named("Address".to_string());
```

### PrimitiveKind

Все поддерживаемые примитивные типы:

| PrimitiveKind | Rust Type | Описание |
|---------------|-----------|----------|
| `String` | `String` | Строка |
| `I8` - `I128` | `i8` - `i128` | Знаковые целые |
| `U8` - `U128` | `u8` - `u128` | Беззнаковые целые |
| `F32`, `F64` | `f32`, `f64` | Числа с плавающей точкой |
| `Bool` | `bool` | Булево значение |
| `Char` | `char` | Символ |
| `DateTime` | `chrono::DateTime<Utc>` | Дата и время |
| `Uuid` | `uuid::Uuid` | UUID |
| `Decimal` | `rust_decimal::Decimal` | Точное десятичное число |
| `Json` | `serde_json::Value` | Произвольный JSON |

### FieldConstraints

Ограничения валидации для полей:

```rust
use unistructgen_core::FieldConstraints;

let constraints = FieldConstraints {
    min_length: Some(3),           // Минимальная длина строки
    max_length: Some(100),         // Максимальная длина
    min_value: Some(0.0),          // Минимальное значение числа
    max_value: Some(1000.0),       // Максимальное значение
    pattern: Some(r"^\w+$".to_string()), // Regex паттерн
    format: Some("email".to_string()),   // Формат (email, url, etc.)
};
```

---

## 🎨 Unified API

Эргономичный Builder API для программного создания структур.

### StructGen

```rust
use unistructgen_core::{StructGen, FieldType, FieldBuilder};

let code = StructGen::new()
    .name("Order")
    .doc("Заказ в системе")

    // Простое поле
    .field("id", FieldType::Uuid)

    // Опциональное поле
    .field_optional("discount", FieldType::Decimal)

    // Поле с кастомизацией через билдер
    .field_with(|f| f
        .doc("Email покупателя")
        .format("email")
        .length(5, 255),
        "customer_email", FieldType::String
    )

    // Вложенный тип
    .field("items", FieldType::vec(FieldType::named("OrderItem")))

    // Derives
    .with_serde()
    .with_default()
    .with_eq()

    // Генерация
    .generate()?;

println!("{}", code);
```

### EnumGen

```rust
use unistructgen_core::EnumGen;

let code = EnumGen::new()
    .name("PaymentMethod")
    .doc("Способ оплаты")
    .variant("CreditCard")
    .variant("DebitCard")
    .variant_with_rename("BankTransfer", "bank_transfer")
    .variant_with_doc("Crypto", "Криптовалюта (BTC, ETH)")
    .with_serde()
    .generate()?;
```

### ModuleGen

Объединение нескольких типов в модуль:

```rust
use unistructgen_core::{ModuleGen, StructGen, EnumGen, FieldType};

let code = ModuleGen::new("ecommerce")
    .add_struct(
        StructGen::new()
            .name("Product")
            .field("id", FieldType::I64)
            .field("name", FieldType::String)
            .field("price", FieldType::Decimal)
    )
    .add_struct(
        StructGen::new()
            .name("Order")
            .field("products", FieldType::vec(FieldType::named("Product")))
            .field("status", FieldType::named("OrderStatus"))
    )
    .add_enum(
        EnumGen::new()
            .name("OrderStatus")
            .variant("Pending")
            .variant("Shipped")
            .variant("Delivered")
    )
    .generate()?;
```

### FieldBuilder

Детальная настройка полей:

```rust
use unistructgen_core::{FieldBuilder, FieldType};

let field = FieldBuilder::new("age", FieldType::I32)
    .optional()                    // Option<i32>
    .doc("Возраст пользователя")
    .range(0.0, 150.0)            // #[validate(range(min = 0, max = 150))]
    .build();

let email_field = FieldBuilder::new("email", FieldType::String)
    .doc("Email пользователя")
    .format("email")              // #[validate(email)]
    .length(5, 255)               // #[validate(length(min = 5, max = 255))]
    .rename("user_email")         // #[serde(rename = "user_email")]
    .build();
```

### FieldType

Упрощённое описание типов:

```rust
use unistructgen_core::FieldType;

// Примитивы
FieldType::String
FieldType::I64
FieldType::Bool
FieldType::DateTime
FieldType::Uuid
FieldType::Decimal
FieldType::Json

// Составные типы
FieldType::optional(FieldType::String)         // Option<String>
FieldType::vec(FieldType::I32)                 // Vec<i32>
FieldType::map(FieldType::String, FieldType::I64)  // HashMap<String, i64>
FieldType::named("Address")                    // Address (ссылка на тип)

// Вложенные
FieldType::vec(FieldType::optional(FieldType::named("Item")))  // Vec<Option<Item>>
```

---

## 📐 Трейты

### Parser

Интерфейс для всех парсеров:

```rust
use unistructgen_core::{Parser, IRModule, ParserMetadata};

pub trait Parser {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Парсинг входных данных в IR
    fn parse(&mut self, input: &str) -> Result<IRModule, Self::Error>;

    /// Имя парсера
    fn name(&self) -> &'static str;

    /// Поддерживаемые расширения файлов
    fn extensions(&self) -> &[&'static str];

    /// Быстрая валидация без полного парсинга (опционально)
    fn validate(&self, input: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Метаданные парсера (опционально)
    fn metadata(&self) -> ParserMetadata {
        ParserMetadata::default()
    }
}
```

**Реализация своего парсера:**

```rust
use unistructgen_core::{Parser, IRModule, IRStruct, IRType};

struct TomlParser;

#[derive(Debug, thiserror::Error)]
enum TomlError {
    #[error("TOML parse error: {0}")]
    Parse(String),
}

impl Parser for TomlParser {
    type Error = TomlError;

    fn parse(&mut self, input: &str) -> Result<IRModule, Self::Error> {
        // Логика парсинга TOML → IR
        let mut module = IRModule::new("config".to_string());
        // ...
        Ok(module)
    }

    fn name(&self) -> &'static str { "TOML" }
    fn extensions(&self) -> &[&'static str] { &["toml"] }
}
```

### CodeGenerator

Интерфейс для генераторов кода:

```rust
use unistructgen_core::{CodeGenerator, IRModule, GeneratorMetadata};

pub trait CodeGenerator {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Генерация кода из IR
    fn generate(&self, module: &IRModule) -> Result<String, Self::Error>;

    /// Целевой язык
    fn language(&self) -> &'static str;

    /// Расширение файлов
    fn file_extension(&self) -> &str;

    /// Валидация IR перед генерацией (опционально)
    fn validate(&self, module: &IRModule) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Форматирование сгенерированного кода (опционально)
    fn format(&self, code: String) -> Result<String, Self::Error> {
        Ok(code)
    }
}
```

**Расширенные методы (CodeGeneratorExt):**

```rust
use unistructgen_core::CodeGeneratorExt;

let generator = RustRenderer::new(options);

// Валидация + генерация
let code = generator.generate_validated(&module)?;

// Генерация + форматирование
let code = generator.generate_formatted(&module)?;

// Всё вместе
let code = generator.generate_complete(&module)?;
```

### IRTransformer

Интерфейс для трансформации IR:

```rust
use unistructgen_core::{IRTransformer, IRModule, TransformError};

pub trait IRTransformer: Send + Sync {
    /// Имя трансформера
    fn name(&self) -> &str;

    /// Трансформация модуля
    fn transform(&self, module: IRModule) -> Result<IRModule, TransformError>;

    /// Описание (опционально)
    fn description(&self) -> Option<&str> {
        None
    }
}
```

---

## 🔄 Трансформеры

Встроенные трансформеры для модификации IR.

### FieldOptionalizer

Делает все поля опциональными (заворачивает в `Option<T>`):

```rust
use unistructgen_core::transformer::FieldOptionalizer;

let transformer = FieldOptionalizer::new();
let transformed_module = transformer.transform(module)?;

// До: pub id: i64
// После: pub id: Option<i64>
```

### DocCommentAdder

Автоматически генерирует документацию:

```rust
use unistructgen_core::transformer::DocCommentAdder;

// Добавляет документацию к типам и полям
let transformer = DocCommentAdder::new();

// Только к типам (без полей)
let transformer = DocCommentAdder::types_only();

// До:
// pub struct UserProfile { pub user_name: String }

// После:
// /// Represents a User Profile
// pub struct UserProfile {
//     /// user name
//     pub user_name: String
// }
```

### TypeDeduplicator

Удаляет дублирующиеся структуры:

```rust
use unistructgen_core::transformer::TypeDeduplicator;

let transformer = TypeDeduplicator::new();

// Если есть User1 и User2 с идентичными полями —
// останется только первый
```

### FieldRenamer

Переименовывает поля по маппингу:

```rust
use unistructgen_core::transformer::FieldRenamer;
use std::collections::HashMap;

let mut mappings = HashMap::new();
mappings.insert("id".to_string(), "identifier".to_string());
mappings.insert("name".to_string(), "full_name".to_string());

let transformer = FieldRenamer::new(mappings);
```

### Создание своего трансформера

```rust
use unistructgen_core::{IRTransformer, IRModule, IRType, TransformError};

struct AddCreatedAtField;

impl IRTransformer for AddCreatedAtField {
    fn name(&self) -> &str { "AddCreatedAtField" }

    fn description(&self) -> Option<&str> {
        Some("Добавляет поле created_at ко всем структурам")
    }

    fn transform(&self, mut module: IRModule) -> Result<IRModule, TransformError> {
        for ty in &mut module.types {
            if let IRType::Struct(ref mut s) = ty {
                let field = IRField::new(
                    "created_at".to_string(),
                    IRTypeRef::Primitive(PrimitiveKind::DateTime)
                );
                s.add_field(field);
            }
        }
        Ok(module)
    }
}
```

---

## ⚙️ Pipeline

Композируемый конвейер обработки данных.

### Базовое использование

```rust
use unistructgen_core::Pipeline;
use unistructgen_json_parser::{JsonParser, ParserOptions};
use unistructgen_codegen::{RustRenderer, RenderOptions};

let parser = JsonParser::new(ParserOptions::default());
let generator = RustRenderer::new(RenderOptions::default());

let mut pipeline = Pipeline::new(parser, generator);

let json = r#"{"id": 1, "name": "Alice"}"#;
let code = pipeline.execute(json)?;
```

### С трансформерами

```rust
use unistructgen_core::{Pipeline, transformer::*};

let mut pipeline = Pipeline::new(parser, generator)
    .add_transformer(Box::new(FieldOptionalizer::new()))
    .add_transformer(Box::new(DocCommentAdder::new()))
    .add_transformer(Box::new(TypeDeduplicator::new()));

// Трансформеры применяются в порядке добавления
```

### PipelineBuilder

Альтернативный fluent-синтаксис:

```rust
use unistructgen_core::PipelineBuilder;

let pipeline = PipelineBuilder::new()
    .parser(json_parser)
    .generator(rust_renderer)
    .transformer(Box::new(FieldOptionalizer::new()))
    .transformer(Box::new(DocCommentAdder::new()))
    .build();
```

### Интроспекция Pipeline

```rust
// Количество трансформеров
let count = pipeline.transformer_count();

// Имена трансформеров
let names = pipeline.transformer_names();
// ["FieldOptionalizer", "DocCommentAdder"]

// Доступ к парсеру
let parser_name = pipeline.parser().name();

// Доступ к генератору
let language = pipeline.generator().language();
```

---

## 🔌 Plugin System

Расширяемая система плагинов с хуками на разных этапах обработки.

### Интерфейс Plugin

```rust
use unistructgen_core::{Plugin, PluginError, IRModule};

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> Option<&str> { None }

    /// Инициализация плагина
    fn initialize(&mut self) -> Result<(), PluginError>;

    /// Хук перед парсингом (модификация входных данных)
    fn before_parse(&mut self, input: &str) -> Result<String, PluginError> {
        Ok(input.to_string())
    }

    /// Хук после парсинга (модификация IR)
    fn after_parse(&mut self, module: IRModule) -> Result<IRModule, PluginError> {
        Ok(module)
    }

    /// Хук после генерации (модификация кода)
    fn after_generate(&mut self, code: String) -> Result<String, PluginError> {
        Ok(code)
    }

    /// Завершение работы
    fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}
```

### Встроенные плагины

**HeaderPlugin** — добавляет заголовок к сгенерированному коду:

```rust
use unistructgen_core::plugin::HeaderPlugin;

let plugin = HeaderPlugin::new("Generated by MyTool v1.0");
```

**LoggingPlugin** — логирует этапы обработки:

```rust
use unistructgen_core::plugin::LoggingPlugin;

let plugin = LoggingPlugin::new(true); // verbose mode
```

### PluginRegistry

```rust
use unistructgen_core::PluginRegistry;

let mut registry = PluginRegistry::new();

// Регистрация плагинов
registry.register(Box::new(HeaderPlugin::new("// Auto-generated")))?;
registry.register(Box::new(LoggingPlugin::new(true)))?;

// Проверка наличия
assert!(registry.has_plugin("HeaderPlugin"));

// Получение плагина
if let Some(plugin) = registry.get("LoggingPlugin") {
    println!("Version: {}", plugin.version());
}

// Выполнение хуков
let input = registry.before_parse(input)?;
let module = registry.after_parse(module)?;
let code = registry.after_generate(code)?;

// Количество плагинов
println!("Plugins: {}", registry.count());
```

### Создание своего плагина

```rust
use unistructgen_core::{Plugin, PluginError, IRModule, IRType};

struct MetricsPlugin {
    struct_count: usize,
    field_count: usize,
}

impl MetricsPlugin {
    fn new() -> Self {
        Self { struct_count: 0, field_count: 0 }
    }
}

impl Plugin for MetricsPlugin {
    fn name(&self) -> &str { "MetricsPlugin" }
    fn version(&self) -> &str { "1.0.0" }

    fn initialize(&mut self) -> Result<(), PluginError> {
        self.struct_count = 0;
        self.field_count = 0;
        Ok(())
    }

    fn after_parse(&mut self, module: IRModule) -> Result<IRModule, PluginError> {
        for ty in &module.types {
            if let IRType::Struct(s) = ty {
                self.struct_count += 1;
                self.field_count += s.fields.len();
            }
        }
        println!("Metrics: {} structs, {} fields",
                 self.struct_count, self.field_count);
        Ok(module)
    }
}
```

---

## 👁️ Visitor Pattern

Паттерн для обхода и анализа IR без модификации.

### IRVisitor trait

```rust
use unistructgen_core::{IRVisitor, IRModule, IRStruct, IRField, IRTypeRef};

pub trait IRVisitor {
    fn visit_module(&mut self, module: &mut IRModule) { /* ... */ }
    fn visit_struct(&mut self, struct_: &mut IRStruct) { /* ... */ }
    fn visit_field(&mut self, field: &mut IRField) { /* ... */ }
    fn visit_type_ref(&mut self, type_ref: &mut IRTypeRef) { /* ... */ }
    // ...
}
```

### Встроенные визиторы

**StructNameCollector** — собирает имена всех структур:

```rust
use unistructgen_core::{IRVisitor, visitor::StructNameCollector};

let mut collector = StructNameCollector::new();
collector.visit_module(&mut module);

let names = collector.names(); // ["User", "Address", "Order"]
```

**FieldCounter** — подсчитывает поля:

```rust
use unistructgen_core::visitor::FieldCounter;

let mut counter = FieldCounter::new();
counter.visit_module(&mut module);

println!("Total fields: {}", counter.count());
```

**PrimitiveTypeCollector** — собирает используемые примитивные типы:

```rust
use unistructgen_core::visitor::PrimitiveTypeCollector;

let mut collector = PrimitiveTypeCollector::new();
collector.visit_module(&mut module);

let types = collector.types();
// HashSet {String, I64, DateTime, Uuid}
```

**IRValidator** — валидирует структуру IR:

```rust
use unistructgen_core::visitor::IRValidator;

let mut validator = IRValidator::new();
validator.visit_module(&mut module);

if validator.has_errors() {
    for error in validator.errors() {
        eprintln!("Validation error: {}", error);
    }
}
```

### Создание своего визитора

```rust
use unistructgen_core::{IRVisitor, IRStruct, IRField, walk_struct};

struct RequiredFieldChecker {
    missing_required: Vec<String>,
}

impl IRVisitor for RequiredFieldChecker {
    fn visit_struct(&mut self, s: &mut IRStruct) {
        let has_id = s.fields.iter().any(|f| f.name == "id");
        if !has_id {
            self.missing_required.push(format!(
                "Struct '{}' is missing required 'id' field",
                s.name
            ));
        }

        // Важно: не забыть обойти дочерние элементы
        walk_struct(self, s);
    }
}
```

---

## 📝 Примеры

### Полный цикл: JSON → Rust

```rust
use unistructgen_core::{Pipeline, transformer::*};
use unistructgen_json_parser::{JsonParser, ParserOptions};
use unistructgen_codegen::{RustRenderer, RenderOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{
        "user": {
            "id": 12345,
            "name": "Alice",
            "email": "alice@example.com",
            "created_at": "2024-01-15T10:30:00Z"
        },
        "orders": [
            {"id": 1, "amount": 99.99}
        ]
    }"#;

    let parser = JsonParser::new(ParserOptions {
        struct_name: "ApiResponse".to_string(),
        derive_serde: true,
        ..Default::default()
    });

    let generator = RustRenderer::new(RenderOptions::default());

    let mut pipeline = Pipeline::new(parser, generator)
        .add_transformer(Box::new(DocCommentAdder::new()));

    let code = pipeline.execute(json)?;
    println!("{}", code);

    Ok(())
}
```

### Программное создание сложных типов

```rust
use unistructgen_core::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // E-commerce модели
    let product = StructGen::new()
        .name("Product")
        .doc("Товар в каталоге")
        .field("id", FieldType::Uuid)
        .field("sku", FieldType::String)
        .field("name", FieldType::String)
        .field("price", FieldType::Decimal)
        .field_optional("description", FieldType::String)
        .field("categories", FieldType::vec(FieldType::String));

    let order_item = StructGen::new()
        .name("OrderItem")
        .field("product_id", FieldType::Uuid)
        .field_with(|f| f.range(1.0, 1000.0), "quantity", FieldType::I32)
        .field("unit_price", FieldType::Decimal);

    let order_status = EnumGen::new()
        .name("OrderStatus")
        .variant("Draft")
        .variant("Pending")
        .variant("Paid")
        .variant("Shipped")
        .variant("Delivered")
        .variant("Cancelled");

    let order = StructGen::new()
        .name("Order")
        .doc("Заказ клиента")
        .field("id", FieldType::Uuid)
        .field("customer_id", FieldType::Uuid)
        .field("status", FieldType::named("OrderStatus"))
        .field("items", FieldType::vec(FieldType::named("OrderItem")))
        .field("created_at", FieldType::DateTime)
        .field_optional("shipped_at", FieldType::DateTime);

    let code = ModuleGen::new("ecommerce")
        .add_struct(product)
        .add_struct(order_item)
        .add_struct(order)
        .add_enum(order_status)
        .generate()?;

    println!("{}", code);

    Ok(())
}
```

---

## 🔗 Связанные модули

- [unistructgen-codegen](../codegen/README.md) — Rust-генератор
- [unistructgen-json-parser](../parsers/json_parser/README.md) — JSON-парсер
- [unistructgen-markdown-parser](../parsers/markdown_parser/README.md) — Markdown-парсер
- [unistructgen-openapi-parser](../parsers/openapi_parser/README.md) — OpenAPI-парсер
- [unistructgen-macro](../proc-macro/README.md) — Процедурные макросы

---

## 📜 Лицензия

MIT или Apache-2.0 — на ваш выбор.
