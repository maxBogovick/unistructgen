# Руководство: Создание профессиональных AI-инструментов на Rust с MCP

Это руководство — пошаговый план для создания **Model Context Protocol (MCP)** сервера с использованием `unistructgen`.

**Цель:** Научиться превращать обычные Rust-функции в инструменты, которыми может управлять Искусственный Интеллект (например, Claude 3.5 Sonnet в приложении Claude Desktop, IDE Cursor или Windsurf), без написания сложного кода для API.

---

## 1. Введение: Зачем нам это нужно?

**Model Context Protocol (MCP)** — это открытый стандарт от Anthropic, который позволяет подключать к LLM внешние данные и инструменты.

Раньше, чтобы дать ИИ доступ к базе данных или файловой системе, нужно было:
1. Писать HTTP-сервер.
2. Описывать JSON-схемы вручную.
3. Писать логику обработки запросов.

С библиотекой `unistructgen-mcp` это делается **декларативно**. Вы пишете функцию, вешаете макрос `#[ai_tool]`, и сервер готов.

---

## 2. Подготовка окружения

Вам понадобится:
1. **Rust** (версия 1.70+).
2. **Claude Desktop** (приложение для Mac/Windows) — это будет наш "клиент", который будет вызывать инструменты.
3. Любимая IDE (RustRover, VS Code).

### Сетевые настройки compile-time fetch (опционально)

Если вы используете макросы, которые делают сетевые запросы при компиляции (например, `struct_from_external_api!` или `openapi_to_rust!` с `url`), можно управлять поведением через переменные окружения:

- `UNISTRUCTGEN_FETCH_OFFLINE=1` — запрет сети, только кеш
- `UNISTRUCTGEN_FETCH_CACHE=0` — отключить кеш
- `UNISTRUCTGEN_FETCH_CACHE_DIR=/path` — путь к кешу
- `UNISTRUCTGEN_FETCH_TIMEOUT_MS=60000` — таймаут (мс)

### Шаг 2.1: Создание проекта

Создадим новый бинарный проект.

```bash
cargo new my-mcp-server
cd my-mcp-server
```

### Шаг 2.2: Настройка зависимостей

В `Cargo.toml` добавьте следующие зависимости. Мы используем локальные пути для `unistructgen`, но в реальном проекте это были бы версии с crates.io или git.

```toml
[package]
name = "my-mcp-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# Основные компоненты unistructgen
unistructgen-core = { path = "../core" }        # Типы и Registry
unistructgen-macro = { path = "../proc-macro" } # Макрос #[ai_tool]
unistructgen-mcp = { path = "../mcp" }          # Сервер MCP

# Асинхронный рантайм
tokio = { version = "1.0", features = ["full"] }

# Сериализация (нужна для работы макросов под капотом)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Обработка ошибок
anyhow = "1.0"

# Логирование (КРИТИЧНО ВАЖНО: stdout занят протоколом MCP)
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

## 3. Написание Инструментов (Tools)

Мы создадим "умный системный монитор". Инструменты будут позволять ИИ читать файлы и выполнять базовые расчеты.

Откройте `src/main.rs`.

### Шаг 3.1: Импорты и настройка

```rust
use unistructgen_macro::ai_tool;
use unistructgen_core::{ToolRegistry, Context};
use unistructgen_mcp::serve_stdio;
use std::sync::Arc;
use std::fs;
use std::path::Path;

// Важно: настройки логирования должны идти в stderr!
fn init_logging() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // <-- КЛЮЧЕВОЙ МОМЕНТ
        .with_env_filter("info")
        .init();
}
```

### Шаг 3.2: Простые инструменты

Создадим инструмент для чтения файла. Макрос `#[ai_tool]` автоматически сгенерирует JSON Schema, понятную для LLM.

```rust
/// Читает содержимое текстового файла по указанному пути.
/// Используйте этот инструмент, когда нужно проанализировать код или конфиг.
#[ai_tool]
fn read_file(path: String) -> Result<String, String> {
    // Логируем действие (увидим это в логах Claude Desktop)
    tracing::info!("Запрошено чтение файла: {}", path);

    let path = Path::new(&path);
    if !path.exists() {
        return Err(format!("Файл не найден: {:?}", path));
    }

    fs::read_to_string(path)
        .map_err(|e| format!("Ошибка чтения файла: {}", e))
}

/// Складывает два числа. Полезно для проверки математических способностей модели.
#[ai_tool]
fn add_numbers(a: i64, b: i64) -> i64 {
    a + b
}
```

### Шаг 3.3: Продвинутый инструмент с Контекстом (Dependency Injection)

Допустим, у нас есть глобальная конфигурация или база данных, которую мы не хотим передавать в каждый запрос от LLM, но она нужна внутри функции.

Используем атрибут `#[context]`.

```rust
// Наша "база данных" или конфиг
#[derive(Clone)]
struct AppConfig {
    root_dir: String,
}

/// Получает список файлов в разрешенной директории.
#[ai_tool]
fn list_files(
    #[context] config: AppConfig, // <-- Это поле внедряется автоматически, LLM его не видит
    sub_path: Option<String>      // <-- Это поле заполняет LLM
) -> Result<String, String> {
    let target_dir = match sub_path {
        Some(p) => format!("{}/{}", config.root_dir, p),
        None => config.root_dir.clone(),
    };

    tracing::info!("Листинг директории: {}", target_dir);

    let entries = fs::read_dir(&target_dir)
        .map_err(|e| format!("Ошибка доступа: {}", e))?;

    let mut filenames = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            filenames.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    Ok(filenames.join("\n"))
}
```

---

## 4. Сборка Сервера

Теперь соберем все вместе в `main`.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Инициализация логов (строго в stderr)
    init_logging();

    // 2. Реестр инструментов
    let mut registry = ToolRegistry::new();
    
    // Регистрируем сгенерированные структуры (Macro создает StructName + Tool)
    registry.register(ReadFileTool);
    registry.register(AddNumbersTool);
    registry.register(ListFilesTool);

    // 3. Создаем контекст
    let mut context = Context::new();
    // Внедряем наш конфиг
    context.insert(AppConfig {
        root_dir: ".".to_string(), // Разрешаем доступ к текущей папке
    });

    tracing::info!("MCP сервер запускается...");

    // 4. Запуск сервера на stdio (стандартный ввод/вывод)
    // Это основной метод общения для локальных агентов
    serve_stdio(Arc::new(registry), context).await?;

    Ok(())
}
```

---

## 5. Компиляция и Проверка

Скомпилируйте проект в release-режиме, чтобы он работал быстро.

```bash
car go build --release
```

Получим путь к бинарнику:
`target/release/my-mcp-server` (или `.exe` на Windows).

**Важное правило:**
Никогда не используйте `println!` внутри инструментов при работе через stdio! Протокол MCP использует `stdout` для общения. Любой лишний текст сломает JSON-поток. Используйте `eprintln!` или `tracing` (который настроен на stderr).

---

## 6. Подключение к Claude Desktop

Это самый интересный этап. Мы научим официальное приложение Claude использовать ваш Rust-код.

### Шаг 6.1: Конфигурация

Найдите файл конфигурации Claude:
- **macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

Откройте его и добавьте ваш сервер:

```json
{
  "mcpServers": {
    "my-rust-tools": {
      "command": "/ABSOLUTE/PATH/TO/YOUR/PROJECT/target/release/my-mcp-server",
      "args": []
    }
  }
}
```
*Замените путь на реальный абсолютный путь к вашему скомпилированному файлу.*

### Шаг 6.2: Запуск

1. Перезапустите Claude Desktop полностью.
2. Обратите внимание на иконку "вилки" (Tools) справа в поле ввода.
3. Вы должны увидеть: `my-rust-tools` с тремя инструментами: `read_file`, `add_numbers`, `list_files`.

---

## 7. Демонстрация (Сценарий для видео)

Теперь вы можете общаться с Claude так, будто он — это вы в терминале.

**Запрос 1:**
> "Пожалуйста, посмотри, какие файлы есть в текущей директории."

**Что происходит:**
1. Claude видит инструмент `list_files`.
2. Он понимает, что `config` ему передавать не надо (это делает сервер), а `sub_path` опционален.
3. Он отправляет запрос MCP: `call_tool("list_files", {})`.
4. Ваша Rust-программа выполняется, берет путь из Context, читает диск и возвращает список.
5. Claude показывает результат.

**Запрос 2:**
> "Прочитай файл Cargo.toml и скажи мне версию пакета."

**Что происходит:**
1. Claude вызывает `read_file(path="Cargo.toml")`.
2. Rust читает файл.
3. Claude анализирует текст и отвечает: "Версия пакета 0.1.0".

---

## 8. Чек-лист для Профессионала

1. **Типизация:** Используйте `Result<T, E>`. Если инструмент возвращает `Err`, LLM увидит ошибку и может попытаться исправить параметры запроса самостоятельно.
2. **Безопасность:** Вы даете ИИ доступ к исполнению кода. В функции `read_file` стоит добавить проверку, чтобы нельзя было выйти за пределы разрешенной директории (например, прочитать `/etc/passwd`).
3. **Описание (Doc comments):** Качество работы ИИ напрямую зависит от того, как хорошо вы описали функцию в `///`.
   - Плохо: `/// Читает файл`
   - Хорошо: `/// Читает содержимое файла. Используйте этот инструмент, чтобы проанализировать исходный код или конфигурацию перед внесением изменений.`
4. **Context:** Всегда используйте `Context` для передачи подключений к БД и API-ключей. Никогда не заставляйте LLM "галлюцинировать" креды.

---

## 9. Заключение

Вы только что создали высокопроизводительный, типобезопасный сервер инструментов для ИИ на Rust.

**Преимущества этого подхода:**
- **Скорость:** Rust работает мгновенно.
- **Надежность:** Строгая типизация гарантирует, что если аргументы пришли, они корректны.
- **Удобство:** Макросы делают всю грязную работу по генерации JSON Schema.

Теперь ваш ИИ-ассистент не ограничен своим "контекстным окном" — он может взаимодействовать с реальным миром через ваш код.
