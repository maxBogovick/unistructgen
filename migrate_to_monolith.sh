#!/bin/bash
set -e # Остановить скрипт при любой ошибке

echo "🚀 Начинаем миграцию на монолитную структуру (v9 - Safe Mode)..."

# 1. Создаем новую ветку
echo "🌿 Работаем в ветке 'refactor/monolith-migration'..."
git checkout refactor/monolith-migration 2>/dev/null || git checkout -b refactor/monolith-migration

# 2. Подготовка структуры папок
echo "📁 Создаем новую структуру папок..."
mkdir -p src/core
mkdir -p src/codegen
mkdir -p src/llm
mkdir -p src/agent
mkdir -p src/mcp
mkdir -p src/parsers/json
mkdir -p src/parsers/sql
mkdir -p src/parsers/markdown
mkdir -p src/parsers/openapi
mkdir -p src/parsers/graphql
mkdir -p src/parsers/env

# Папки для бэкапов
mkdir -p tests_backup
mkdir -p benches_backup
mkdir -p build_scripts_backup
mkdir -p assets_backup

# 3. Функция для перемещения кода
move_module() {
    local src_dir=$1
    local dest_dir=$2
    local module_name=$(basename $src_dir)

    echo "Processing $module_name..."

    # Проверка существования источника
    if [ ! -d "$src_dir/src" ]; then
        echo "⚠️  Пропуск: $src_dir/src не найдена (возможно, уже перемещена)"
        return
    fi

    # 1. Перенос исходного кода (используем cp -R src/. для надежности)
    # Используем cp -R для рекурсивного копирования содержимого
    cp -R "$src_dir/src/." "$dest_dir/"

    # Проверка, что копирование прошло успешно
    if [ -z "$(ls -A "$dest_dir")" ]; then
        echo "❌ ОШИБКА: Папка $dest_dir пуста после копирования из $src_dir/src!"
        exit 1
    fi

    # Если был lib.rs, он становится mod.rs
    if [ -f "$dest_dir/lib.rs" ]; then
        mv "$dest_dir/lib.rs" "$dest_dir/mod.rs"
    fi
    echo "   ✅ Код перемещен в $dest_dir"

    # 2. Бэкап тестов
    if [ -d "$src_dir/tests" ]; then
        local test_dest="tests_backup/$module_name"
        mkdir -p "$test_dest"
        cp -R "$src_dir/tests/." "$test_dest/"
        echo "   📦 Тесты сохранены в $test_dest"
    fi

    # 3. Бэкап бенчмарков
    if [ -d "$src_dir/benches" ]; then
        local bench_dest="benches_backup/$module_name"
        mkdir -p "$bench_dest"
        cp -R "$src_dir/benches/." "$bench_dest/"
        echo "   ⏱️ Бенчмарки сохранены в $bench_dest"
    fi

    # 4. Проверка build.rs
    if [ -f "$src_dir/build.rs" ]; then
        cp "$src_dir/build.rs" "build_scripts_backup/build_$module_name.rs"
        echo "   ⚠️  build.rs сохранен в build_scripts_backup/"
    fi

    # 5. Сохранение прочих файлов
    local assets_dest="assets_backup/$module_name"
    mkdir -p "$assets_dest"
    find "$src_dir" -maxdepth 1 -not -name "src" -not -name "tests" -not -name "benches" -not -name "target" -not -name "Cargo.toml" -not -name "build.rs" -not -name ".*" -not -path "$src_dir" -exec cp -R {} "$assets_dest/" \;

    if [ -z "$(ls -A "$assets_dest")" ]; then
        rmdir "$assets_dest"
    else
        echo "   📂 Дополнительные файлы сохранены в $assets_dest"
    fi
}

# 4. Перемещение файлов
echo "📦 Перемещаем модули..."
move_module "core" "src/core"
move_module "codegen" "src/codegen"
move_module "llm" "src/llm"
move_module "agent" "src/agent"
move_module "mcp" "src/mcp"

# Парсеры - внимательно проверяем пути
echo "📦 Перемещаем парсеры..."
move_module "parsers/json_parser" "src/parsers/json"
move_module "parsers/sql_parser" "src/parsers/sql"
move_module "parsers/markdown_parser" "src/parsers/markdown"
move_module "parsers/openapi_parser" "src/parsers/openapi"
move_module "parsers/graphql_parser" "src/parsers/graphql"
move_module "parsers/env_parser" "src/parsers/env"

# Обработка CLI и Macro
if [ -d "cli" ]; then mv cli unistructgen-cli; fi
if [ -d "proc-macro" ]; then mv proc-macro unistructgen-macro; fi

# 5. Создание файлов структуры
echo "📝 Создаем src/lib.rs и src/parsers/mod.rs..."

# src/lib.rs
cat <<EOF > src/lib.rs
pub mod core;
pub mod codegen;

#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "agent")]
pub mod agent;

#[cfg(feature = "mcp")]
pub mod mcp;

pub mod parsers;

// Prelude: Экспортируем типы из core для удобства
pub use core::*;
EOF

# src/parsers/mod.rs
cat <<EOF > src/parsers/mod.rs
#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "sql")]
pub mod sql;

#[cfg(feature = "openapi")]
pub mod openapi;

#[cfg(feature = "markdown")]
pub mod markdown;

#[cfg(feature = "graphql")]
pub mod graphql;

#[cfg(feature = "env")]
pub mod env;
EOF

# 6. Генерация Cargo.toml файлов
echo "📝 Генерируем Cargo.toml..."

# Собираем список примеров для workspace
EXAMPLES_MEMBERS=""
if [ -d "examples" ]; then
    for d in examples/*/; do
        d=${d%/}
        if [ -f "$d/Cargo.toml" ]; then
            EXAMPLES_MEMBERS="$EXAMPLES_MEMBERS, \"$d\""
        fi
    done
fi

# Root Cargo.toml
cat <<EOF > Cargo.toml
[package]
name = "unistructgen"
version = "0.2.0"
edition = "2021"
authors = ["Maxim"]
description = "A powerful Rust code generator"
license = "MIT OR Apache-2.0"
repository = "https://github.com/maxBogovick/unistructgen"
readme = "README.md"
keywords = ["codegen", "json", "struct", "macro", "api"]
categories = ["development-tools"]

[workspace]
members = ["unistructgen-macro", "unistructgen-cli"$EXAMPLES_MEMBERS]

[features]
default = ["json"]
full = ["json", "markdown", "openapi", "sql", "graphql", "env", "llm", "agent", "mcp"]

# Parsers
json = ["dep:serde_json"]
markdown = ["dep:pulldown-cmark"]
openapi = ["dep:openapiv3", "dep:serde_yaml", "dep:indexmap", "dep:ureq", "dep:regex"]
sql = ["dep:sqlparser"]
graphql = ["dep:graphql-parser"]
env = []

# Advanced
llm = ["dep:reqwest", "dep:async-trait", "dep:dotenv"]
agent = ["llm", "dep:tracing", "dep:uuid", "dep:regex"]
mcp = ["dep:tracing", "dep:axum", "dep:tower-http", "dep:dashmap", "dep:tokio-stream", "dep:uuid"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
futures-util = "0.3"

# Optional
serde_json = { version = "1.0", optional = true }
pulldown-cmark = { version = "0.9", optional = true }
sqlparser = { version = "0.43", optional = true }
graphql-parser = { version = "0.4", optional = true }
openapiv3 = { version = "2.0", features = ["skip_serializing_defaults"], optional = true }
serde_yaml = { version = "0.9", optional = true }
indexmap = { version = "2.0", optional = true }
ureq = { version = "2.9", features = ["json"], optional = true }
regex = { version = "1.10", optional = true }
reqwest = { version = "0.11", features = ["json", "stream"], optional = true }
async-trait = { version = "0.1", optional = true }
dotenv = { version = "0.15", optional = true }
tracing = { version = "0.1", optional = true }
uuid = { version = "1.0", features = ["v4"], optional = true }
axum = { version = "0.7", optional = true }
tower-http = { version = "0.5", features = ["cors"], optional = true }
dashmap = { version = "5.5", optional = true }
tokio-stream = { version = "0.1.18", optional = true }
tokio = { version = "1.0", features = ["full"], optional = true }

[dev-dependencies]
tokio = { version = "1.0", features = ["macros", "rt"] }
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.10"
pretty_assertions = "1.4"
validator = { version = "0.18", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
EOF

# Macro Cargo.toml
cat <<EOF > unistructgen-macro/Cargo.toml
[package]
name = "unistructgen-macro"
version = "0.2.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[lib]
proc-macro = true

[dependencies]
unistructgen = { path = "../", features = ["full"] }
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
proc-macro2 = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
ureq = { version = "2.10", features = ["json"] }
EOF

# CLI Cargo.toml
cat <<EOF > unistructgen-cli/Cargo.toml
[package]
name = "unistructgen-cli"
version = "0.2.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[[bin]]
name = "unistructgen"
path = "src/main.rs"

[dependencies]
unistructgen = { path = "../", features = ["full"] }
anyhow = "1.0"
clap = { version = "4.4", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
colored = "2.1"
inquire = "0.7"
dotenv = "0.15"
ureq = "2.9"
openapiv3 = { version = "2.0", features = ["skip_serializing_defaults"] }
indexmap = "2.2"
EOF

# 7. Магия SED: Обновление импортов
echo "🪄  Обновляем импорты..."

replace_in_files() {
    local search=$1
    local replace=$2
    local dir=$3
    if [ -d "$dir" ]; then
        # Используем crate:: (с двоеточием), чтобы не задеть pub(crate)
        find "$dir" -name "*.rs" -type f -exec sed -i '' "s|$search|$replace|g" {} +
    fi
}

# А. Исправление ВНУТРЕННИХ ссылок (crate:: -> crate::module::)
replace_in_files "crate::" "crate::core::" "src/core"
replace_in_files "crate::" "crate::codegen::" "src/codegen"
replace_in_files "crate::" "crate::llm::" "src/llm"
replace_in_files "crate::" "crate::agent::" "src/agent"
replace_in_files "crate::" "crate::mcp::" "src/mcp"

replace_in_files "crate::" "crate::parsers::json::" "src/parsers/json"
replace_in_files "crate::" "crate::parsers::sql::" "src/parsers/sql"
replace_in_files "crate::" "crate::parsers::markdown::" "src/parsers/markdown"
replace_in_files "crate::" "crate::parsers::openapi::" "src/parsers/openapi"
replace_in_files "crate::" "crate::parsers::graphql::" "src/parsers/graphql"
replace_in_files "crate::" "crate::parsers::env::" "src/parsers/env"

# Б. Исправление МЕЖМОДУЛЬНЫХ ссылок (unistructgen_core -> crate::core)
replace_in_files "unistructgen_core" "crate::core" "src"
replace_in_files "unistructgen_codegen" "crate::codegen" "src"
replace_in_files "unistructgen_llm" "crate::llm" "src"
replace_in_files "unistructgen_agent" "crate::agent" "src"
replace_in_files "unistructgen_mcp" "crate::mcp" "src"

replace_in_files "unistructgen_json_parser" "crate::parsers::json" "src"
replace_in_files "unistructgen_sql_parser" "crate::parsers::sql" "src"
replace_in_files "unistructgen_markdown_parser" "crate::parsers::markdown" "src"
replace_in_files "unistructgen_openapi_parser" "crate::parsers::openapi" "src"
replace_in_files "unistructgen_graphql_parser" "crate::parsers::graphql" "src"
replace_in_files "unistructgen_env_parser" "crate::parsers::env" "src"

# В. Исправление ссылок в CLI, MACRO и EXAMPLES
for dir in "unistructgen-cli" "unistructgen-macro" "examples"; do
    replace_in_files "unistructgen_core" "unistructgen::core" "$dir"
    replace_in_files "unistructgen_codegen" "unistructgen::codegen" "$dir"
    replace_in_files "unistructgen_llm" "unistructgen::llm" "$dir"
    replace_in_files "unistructgen_agent" "unistructgen::agent" "$dir"
    replace_in_files "unistructgen_mcp" "unistructgen::mcp" "$dir"

    replace_in_files "unistructgen_json_parser" "unistructgen::parsers::json" "$dir"
    replace_in_files "unistructgen_sql_parser" "unistructgen::parsers::sql" "$dir"
    replace_in_files "unistructgen_markdown_parser" "unistructgen::parsers::markdown" "$dir"
    replace_in_files "unistructgen_openapi_parser" "unistructgen::parsers::openapi" "$dir"
    replace_in_files "unistructgen_graphql_parser" "unistructgen::parsers::graphql" "$dir"
    replace_in_files "unistructgen_env_parser" "unistructgen::parsers::env" "$dir"
done

# 8. Исправление Cargo.toml в примерах (examples/)
echo "🔧 Чиним Cargo.toml в примерах..."
if [ -d "examples" ]; then
    find examples -name "Cargo.toml" -type f | while read toml_file; do
        sed -i '' 's|unistructgen-core|unistructgen|g' "$toml_file"
        sed -i '' 's|path = "../../core"|path = "../../", features = ["full"]|g' "$toml_file"
        sed -i '' '/unistructgen-llm/d' "$toml_file"
        sed -i '' '/unistructgen-agent/d' "$toml_file"
        sed -i '' '/unistructgen-codegen/d' "$toml_file"
        sed -i '' '/unistructgen-.*-parser/d' "$toml_file"
        echo "   -> Обновлен $toml_file"
    done
fi

# 9. Удаление старых папок (ОТКЛЮЧЕНО ДЛЯ БЕЗОПАСНОСТИ)
echo "⚠️  ВНИМАНИЕ: Автоматическое удаление старых папок отключено."
echo "⚠️  Пожалуйста, проверьте, что все файлы перенеслись в src/, и удалите папки core, codegen, parsers и т.д. вручную."
# rm -rf core codegen llm agent mcp parsers

echo "✨ Миграция завершена!"
echo "👉 Запустите 'cargo build', чтобы проверить результат."
