#!/bin/bash
set -e

echo "🔧 Исправляем пути в примерах..."

# Находим все Cargo.toml в папке examples
find examples -name "Cargo.toml" -type f | while read toml_file; do
    echo "Processing $toml_file..."

    # 1. Исправляем путь к макросам: ../../proc-macro -> ../../unistructgen-macro
    sed -i '' 's|path = "../../proc-macro"|path = "../../unistructgen-macro"|g' "$toml_file"

    # 2. На всякий случай, если путь был ../proc-macro (для вложенности 1 уровня)
    sed -i '' 's|path = "../proc-macro"|path = "../unistructgen-macro"|g' "$toml_file"

    # 3. Исправляем зависимость unistructgen-core -> unistructgen (если скрипт миграции пропустил)
    sed -i '' 's|unistructgen-core|unistructgen|g' "$toml_file"

    # 4. Исправляем путь к core: ../../core -> ../../ (корень)
    sed -i '' 's|path = "../../core"|path = "../../", features = ["full"]|g' "$toml_file"

    echo "✅ $toml_file обновлен"
done

echo "✨ Готово! Попробуйте cargo build снова."
