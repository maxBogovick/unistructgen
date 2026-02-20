#!/bin/bash
set -e

echo "🔧 Удаляем вложенные [workspace] из примеров..."

# Находим все Cargo.toml в папке examples
find examples -name "Cargo.toml" -type f | while read toml_file; do
    # Проверяем, есть ли [workspace] в файле
    if grep -q "\[workspace\]" "$toml_file"; then
        echo "Found [workspace] in $toml_file. Removing..."

        # Удаляем строки [workspace] и members = [...]
        # Используем sed для удаления блока [workspace] и следующих строк до пустой строки или следующей секции
        # Но проще просто удалить конкретные строки, так как обычно там только members
        sed -i '' '/\[workspace\]/d' "$toml_file"
        sed -i '' '/members =/d' "$toml_file"

        echo "✅ Removed [workspace] from $toml_file"
    fi
done

echo "✨ Готово! Попробуйте cargo build снова."
