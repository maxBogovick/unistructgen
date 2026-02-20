#!/bin/bash
set -e # Остановить выполнение при ошибке

echo "Начинаем публикацию оставшихся пакетов..."

# 1. Парсеры и MCP
# echo "Publishing unistructgen-graphql-parser..."
# cargo publish -p unistructgen-graphql-parser
# sleep 10
#
# echo "Publishing unistructgen-json-parser..."
# cargo publish -p unistructgen-json-parser
# sleep 10

# echo "Publishing unistructgen-markdown-parser..."
# cargo publish -p unistructgen-markdown-parser
# sleep 10

# echo "Publishing unistructgen-openapi-parser..."
# cargo publish -p unistructgen-openapi-parser
# sleep 10

echo "Publishing unistructgen-sql-parser..."
cargo publish -p unistructgen-sql-parser
sleep 10

echo "Publishing unistructgen-mcp..."
cargo publish -p unistructgen-mcp
sleep 10

# 2. Макросы (зависят от парсеров)
echo "Publishing unistructgen-macro..."
cargo publish -p unistructgen-macro
sleep 10

# 3. CLI (зависит от всего вышеперечисленного)
echo "Publishing unistructgen (CLI)..."
cargo publish -p unistructgen

echo "Все пакеты успешно опубликованы!"
