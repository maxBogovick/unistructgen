# Schema Registry (Experimental)

> Экспериментальная подсистема для управления API‑схемами. Сейчас не интегрирована в основной pipeline UniStructGen.

## ⚠️ Status & Limitations

- Не интегрирована в основной pipeline UniStructGen.
- Генерация кода сейчас использует заглушку.
- Часть заявленных функций находится в разработке.

## 🌟 Возможности (план/частично)

- 🟡 **Schema Management**: хранение и управление API‑спецификациями
- 🟡 **Version Control**: история версий
- 🟡 **Breaking Change Detection**: базовые диффы
- 🟡 **Code Generation**: интеграция с UniStructGen (в работе)
- 🔴 **Multi-Language Generation**: пока не реализовано
- 🔴 **RBAC / Team Management**: пока не реализовано

## 📦 Компоненты

### Server

HTTP API сервер на Axum с PostgreSQL бэкендом (функциональность частично).

```bash
cd server
DATABASE_URL=postgres://localhost/schema_registry cargo run
```

### CLI

Командная строка для работы с registry.

```bash
cd cli
cargo install --path .

# Использование
schema-registry --help
```

### Common

Shared типы и утилиты.

## 🚀 Быстрый старт (для разработчиков)

### 1. Установка

**Prerequisites:**
- Rust 1.70+
- PostgreSQL 14+

**В этом репозитории:**
```bash
cd schema-registry
```

### 2. Настройка базы данных

```bash
# Создайте базу данных
createdb schema_registry

# Установите DATABASE_URL
export DATABASE_URL=postgres://localhost/schema_registry
```

### 3. Запуск сервера

```bash
cd server
cargo run
```

Сервер запустится на `http://localhost:3000`

### 4. Установка CLI

```bash
cd cli
cargo install --path .
```

### 5. Использование (частично)

```bash
# Проверка подключения
schema-registry stats

# Загрузка схемы
schema-registry upload \
  --name user-service \
  --version 1.0.0 \
  --file openapi.yaml \
  --team backend

# Список всех схем
schema-registry list

# Просмотр схемы
schema-registry get user-service

# Сравнение версий
schema-registry diff user-service --from 1.0.0 --to 2.0.0

# Генерация кода (пока заглушка)
schema-registry generate user-service \
  --version 1.0.0 \
  --targets rust \
  --output ./generated
```

## 📖 Документация

### API Endpoints

#### Schemas

```bash
# List schemas
GET /api/schemas?team=backend

# Create schema
POST /api/schemas
{
  "name": "user-service",
  "version": "1.0.0",
  "format": "openapi",
  "team": "backend",
  "content": "...",
  "created_by": "user"
}

# Get schema (latest version)
GET /api/schemas/:name

# Get specific version
GET /api/schemas/:name/:version

# List versions
GET /api/schemas/:name/versions
```

#### Diff

```bash
# Compare versions
POST /api/diff/:name
{
  "from_version": "1.0.0",
  "to_version": "2.0.0"
}
```

#### Generation

```bash
# Generate code
POST /api/generate
{
  "schema_name": "user-service",
  "version": "1.0.0",
  "targets": ["rust", "typescript"]
}
```

#### Teams

```bash
# List teams
GET /api/teams

# Create team
POST /api/teams
{
  "name": "backend",
  "description": "Backend team"
}
```

#### Stats

```bash
# Get statistics
GET /api/stats
```

### CLI Commands

```bash
# Upload schema
schema-registry upload -n NAME -v VERSION -f FILE [-t TEAM]

# List schemas
schema-registry list [--team TEAM]

# Get schema
schema-registry get NAME [--version VERSION]

# List versions
schema-registry versions NAME

# Compare versions
schema-registry diff NAME --from V1 --to V2

# Generate code
schema-registry generate NAME -v VERSION -t TARGETS [-o OUTPUT]

# Show statistics
schema-registry stats

# Team management
schema-registry team list
schema-registry team create NAME [--description DESC]
```

## 🏗️ Архитектура

```
┌──────────────┐
│   CLI        │ (Rust CLI tool)
└──────┬───────┘
       │
┌──────▼───────┐
│   REST API   │ (Axum server)
└──────┬───────┘
       │
   ┌───┴────────────────────┐
   ▼                        ▼
┌────────────┐    ┌──────────────────┐
│ PostgreSQL │    │ Code Generators  │
│ (metadata) │    │ (UniStructGen)   │
└────────────┘    └──────────────────┘
```

### Database Schema

```sql
teams              -- Team management
  ├─ schemas       -- Schema storage
  ├─ team_members  -- Team permissions
  └─ generation_events -- Analytics

schema_versions    -- Version metadata
generated_artifacts -- Cached generated code
```

## 🔧 Configuration

### Server

Environment variables:

```bash
# Database
DATABASE_URL=postgres://localhost/schema_registry

# Server
PORT=3000
```

### CLI

Environment variables:

```bash
# Registry URL
SCHEMA_REGISTRY_URL=http://localhost:3000
```

Config file (`~/.schema-registry/config.toml`):

```toml
[server]
url = "http://localhost:3000"

[auth]
token = "your-api-token"
```

## 📊 Use Cases

### 1. Microservices Management

```bash
# Upload schemas for all services
schema-registry upload -n user-service -v 1.0.0 -f user-api.yaml
schema-registry upload -n payment-service -v 1.0.0 -f payment-api.yaml
schema-registry upload -n order-service -v 1.0.0 -f order-api.yaml

# Generate clients for frontend
schema-registry generate user-service -v 1.0.0 -t typescript -o ./frontend/src/api
```

### 2. API Versioning

```bash
# Upload new version
schema-registry upload -n user-service -v 2.0.0 -f user-api-v2.yaml

# Check for breaking changes
schema-registry diff user-service --from 1.0.0 --to 2.0.0

# Generate migration plan
schema-registry diff user-service --from 1.0.0 --to 2.0.0 --output migration.md
```

### 3. CI/CD Integration

```yaml
# .github/workflows/schema-check.yml
name: Schema Check
on: [pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Upload schema
        run: |
          schema-registry upload \
            --name ${{ github.event.repository.name }} \
            --version ${{ github.sha }} \
            --file openapi.yaml

      - name: Check breaking changes
        run: |
          schema-registry diff ${{ github.event.repository.name }} \
            --from main \
            --to ${{ github.sha }} \
            --fail-on-breaking
```

## 🎯 Roadmap

### Phase 1: MVP (Completed ✅)
- [x] Basic schema management
- [x] Version control
- [x] Breaking change detection
- [x] CLI tool
- [x] REST API

### Phase 2: Enhanced Features
- [ ] Authentication & Authorization
- [ ] Web UI dashboard
- [ ] Webhook notifications
- [ ] Advanced analytics
- [ ] Search & filtering

### Phase 3: Enterprise
- [ ] Multi-tenancy
- [ ] SSO integration
- [ ] Audit logging
- [ ] Advanced RBAC
- [ ] Custom generators
- [ ] On-premise deployment

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under MIT OR Apache-2.0.

## 🙏 Acknowledgments

Built with:
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [SQLx](https://github.com/launchbadge/sqlx) - Database toolkit
- [UniStructGen](https://github.com/unistructgen/unistructgen) - Code generation engine

## 📞 Support

- 📧 Email: support@schema-registry.io
- 💬 Discord: https://discord.gg/schema-registry
- 🐛 Issues: https://github.com/unistructgen/schema-registry/issues

---

**Made with ❤️ by the UniStructGen team**
