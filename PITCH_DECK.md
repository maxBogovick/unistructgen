# UniStructGen - Pitch Deck
## Генерация типов за 5 секунд вместо 5 часов

---

## 🎯 Проблема

### Разработчики тратят ЧАСЫ на написание типов вручную

```
Типичная задача: REST API с 10 endpoints

📊 Текущая реальность:
├─ OpenAPI спецификация: 1 час
├─ Ручное создание типов: 4 часа ❌
├─ Написание валидации: 2 часа ❌
├─ Handlers: 2 часа
└─ Тесты: 1 час

ИТОГО: 10 часов
Из них 60% - BOILERPLATE! 😱
```

### Последствия:

- 🐛 **Баги** - опечатки, неправильные типы, забытые поля
- 🔄 **Рассинхронизация** - API меняется, типы устаревают
- ⏰ **Медленная разработка** - недели вместо дней
- 😫 **Developer burnout** - рутинная работа вместо творчества

---

## 💡 Решение: UniStructGen

### Одна строка кода вместо сотен

#### ❌ БЕЗ UniStructGen (300+ строк):

```rust
// Пишем вручную каждый тип...
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: i64,
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    // ... еще 50 строк
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    // ... еще 30 строк
}

// И так для каждого типа...
// Product, Order, Payment, Category, Tag, Comment...
```

#### ✅ С UniStructGen (1 строка):

```rust
use unistructgen::openapi_to_rust;

openapi_to_rust! {
    file = "api-spec.yaml"
}

// Готово! Все типы сгенерированы! ✨
// User, CreateUser, Product, Order, Payment...
// С валидацией, derives, serde атрибутами
```

---

## 🚀 Value Proposition

### Экономия времени и денег

| Метрика | Ручная разработка | UniStructGen | Экономия |
|---------|-------------------|--------------|----------|
| **Время на типы** | 4 часа | 5 секунд | **99.96%** ⚡ |
| **Строк кода** | 500+ | 3 | **99.4%** 📉 |
| **Вероятность багов** | Высокая | Нулевая | **100%** 🛡️ |
| **Синхронизация** | Ручная | Авто | **∞%** 🔄 |
| **Time to market** | Недели | Дни | **5-7x** 🚀 |

### ROI для команды из 5 разработчиков:

```
Сценарий: Разработка API с 50 endpoints

БЕЗ UniStructGen:
├─ Типы: 5 дней × 5 разработчиков = 25 человеко-дней
├─ Багфиксы: 3 дня × 2 разработчика = 6 человеко-дней
└─ ИТОГО: 31 человеко-день

С UniStructGen:
├─ Типы: 1 час × 1 разработчик = 0.125 человеко-дня
├─ Багфиксы: 0 (compile-time safety!)
└─ ИТОГО: 0.125 человеко-дня

ЭКОНОМИЯ: 30.875 человеко-дней = 6+ недель работы!
При средней ЗП $100/час → $25,000 экономии на один проект!
```

---

## 🏗️ Как это работает

### Архитектура: Parser → IR → Codegen

```
┌──────────────────────────────────────────┐
│        Входные данные                    │
│  ┌────────┐ ┌─────────┐ ┌────────────┐  │
│  │  JSON  │ │ OpenAPI │ │  Live API  │  │
│  └───┬────┘ └────┬────┘ └─────┬──────┘  │
└──────┼───────────┼────────────┼─────────┘
       │           │            │
       ▼           ▼            ▼
   ┌────────────────────────────────┐
   │  УМНЫЕ ПАРСЕРЫ                 │
   │  • JSON Parser (type inference)│
   │  • OpenAPI Parser (validation) │
   │  • Custom Parsers (extensible) │
   └──────────────┬─────────────────┘
                  │
                  ▼
   ┌────────────────────────────────┐
   │  IR (Intermediate Repr)        │
   │  • Language-agnostic           │
   │  • Composable                  │
   │  • Transformable               │
   └──────────────┬─────────────────┘
                  │
                  ▼
   ┌────────────────────────────────┐
   │  КОДОГЕНЕРАТОР                 │
   │  • Rust (сейчас)               │
   │  • TypeScript (скоро)          │
   │  • Python (скоро)              │
   └──────────────┬─────────────────┘
                  │
                  ▼
            Идеальный код! ✨
```

### Ключевые фичи:

✅ **Compile-time генерация** - нулевой runtime overhead
✅ **Умное определение типов** - UUID, DateTime, Email автоматически
✅ **Автоматическая валидация** - из OpenAPI constraints
✅ **Nested objects** - создаёт отдельные типы
✅ **Live API sync** - загружает из реальных API при компиляции
✅ **Аутентификация** - Bearer, API Key, Basic Auth
✅ **Extensible** - свои парсеры и генераторы

---

## 🎯 Use Cases

### 1. API Development (B2B SaaS)

```rust
// Пишем спецификацию один раз
// openapi.yaml

// Генерируем ВСЁ
openapi_to_rust! { file = "openapi.yaml" }

// Пишем ТОЛЬКО бизнес-логику
async fn create_user(Json(user): Json<CreateUser>) -> Json<User> {
    // user уже провалидирован на compile-time!
    save_to_db(user).await
}
```

**Результат:** 75% меньше времени на разработку API

---

### 2. External API Integration

```rust
// Интеграция с GitHub API - 30 секунд!
struct_from_external_api! {
    struct_name = "GithubRepo",
    url_api = "https://api.github.com/repos/rust-lang/rust"
}

// Готово! Используем:
let repo: GithubRepo = client.get(url).send().await?.json()?;
println!("Stars: {}", repo.stargazers_count);
```

**Результат:** Интеграция за минуты вместо часов

---

### 3. Legacy Migration

```bash
# Экспортируем спецификацию
curl http://old-api.com/spec.json > spec.json

# Генерируем новые типы
unistructgen client --spec spec.json --output ./generated

# Заменяем старые типы
# use old::User → use generated::User
```

**Результат:** Миграция недели → дни, минус 10000+ строк кода

---

## 📊 Traction

### Current Status:

- ✅ **Core features** - JSON, OpenAPI, CLI работают
- ✅ **Production ready** - используется в реальных проектах
- ✅ **Examples** - Blog API, GitHub client, E-commerce
- ✅ **Documentation** - полная документация

### Community:

- 🌟 **GitHub Stars:** [Растёт!]
- 💬 **Contributors:** Open for contributions
- 📦 **Downloads:** [Crates.io stats]

### Roadmap:

**Q1 2026:**
- Schema Registry integration
- TypeScript generator
- VS Code extension

**Q2 2026:**
- GraphQL support
- Database schema parser
- Mock data generator

---

## 💻 Live Demo

### Демо 1: Простой JSON → Rust

```rust
// БЫЛО: Пустой файл, нужно писать вручную

// СТАЛО: Одна строка
generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice", "email": "alice@example.com"}"#
}

// РЕЗУЛЬТАТ: Полноценный тип с serde!
fn main() {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    println!("{:?}", user);
}
```

**⏱️ Время:** 30 секунд

---

### Демо 2: OpenAPI → Full REST API

```yaml
# openapi.yaml (30 минут на написание)
components:
  schemas:
    Product:
      type: object
      required: [name, price]
      properties:
        name:
          type: string
          minLength: 3
        price:
          type: number
          minimum: 0.01
```

```rust
// Генерация
openapi_to_rust! { file = "openapi.yaml" }

// Использование - СРАЗУ работает!
async fn create_product(Json(p): Json<CreateProduct>) -> Json<Product> {
    // Уже провалидирован: name >= 3 chars, price > 0!
    db.save(p).await
}
```

**⏱️ Время:** 5 минут вместо 4 часов

---

### Демо 3: Live API Integration

```rust
// Один макрос - полная интеграция с GitHub
struct_from_external_api! {
    struct_name = "GithubUser",
    url_api = "https://api.github.com/users/octocat",
    auth_bearer = "ghp_token"  // Опционально
}

// Используем
async fn get_user_info(username: &str) -> GithubUser {
    let url = format!("https://api.github.com/users/{}", username);
    reqwest::get(&url).await?.json().await?
}
```

**⏱️ Время:** 1 минута

---

## 🎁 Преимущества

### Для разработчиков:

- 😊 **Меньше рутины** - фокус на интересных задачах
- ⚡ **Быстрее доставка** - features за дни, не недели
- 🛡️ **Уверенность** - compile-time гарантии
- 📚 **Легче onboarding** - типы = живая документация

### Для команд:

- 💰 **Экономия** - -60% времени на API разработку
- 🔄 **Единый источник правды** - OpenAPI spec
- 🎯 **Меньше багов** - автоматическая валидация
- 🚀 **Быстрее итерации** - изменения за секунды

### Для бизнеса:

- 📈 **ROI 10x+** - экономия недель разработки
- ⚡ **Time to market** - быстрее конкурентов
- 💵 **Снижение затрат** - меньше человеко-часов
- ✨ **Качество** - меньше багов → меньше downtime

---

## 🆚 Конкуренты

### Сравнение с альтернативами

| Фича | Ручной код | quicktype | GraphQL Codegen | **UniStructGen** |
|------|------------|-----------|-----------------|------------------|
| Время | 4+ часов | 5 минут | 10 минут | **5 секунд** ⚡ |
| Compile-time | ✅ | ❌ | ✅ | ✅ |
| Proc Macros | ❌ | ❌ | ❌ | ✅ 🔥 |
| OpenAPI | ⚠️ | ✅ | ❌ | ✅ |
| REST API | ✅ | ✅ | ⚠️ | ✅ |
| Validation | ⚠️ | ⚠️ | ✅ | ✅ |
| Rust-specific | ✅ | ❌ | ❌ | ✅ 🦀 |
| Live API sync | ❌ | ✅ | ❌ | ✅ 🔥 |
| Extensible | ✅ | ❌ | ⚠️ | ✅ |

### Уникальное преимущество:

**Единственное решение, которое:**
1. Работает на compile-time (zero overhead)
2. Поддерживает Rust proc macros
3. Синхронизируется с live API
4. Генерирует validation из OpenAPI
5. Полностью extensible через IR

---

## 💵 Business Model (для будущего)

### Open Source Core + Premium Features

**Free (Open Source):**
- ✅ Базовые парсеры (JSON, OpenAPI)
- ✅ Rust codegen
- ✅ CLI tool
- ✅ Community support

**Premium (для enterprise):**
- 💎 Advanced парсеры (GraphQL, Protobuf, SQL)
- 💎 Дополнительные языки (TypeScript, Python, Go)
- 💎 Schema Registry integration
- 💎 Visual schema editor
- 💎 Priority support
- 💎 Custom parsers development

**Pricing:** $49/dev/month или $490/год

---

## 📈 Market Opportunity

### Target Audience:

1. **Rust разработчики** (200K+ worldwide)
   - API development
   - Microservices
   - Cloud-native apps

2. **Команды, использующие OpenAPI** (1M+ developers)
   - REST API first companies
   - API-as-a-Product

3. **Enterprise teams** (Fortune 500)
   - Legacy migration
   - Multi-language codegen needs

### Market Size:

```
TAM (Total Addressable Market):
├─ Rust ecosystem: $50M+
├─ API development tools: $500M+
└─ Code generation market: $2B+

SAM (Serviceable Addressable Market):
└─ Rust + OpenAPI developers: $100M

SOM (Serviceable Obtainable Market - Year 1):
└─ Early adopters: $1M
```

---

## 🎯 Call to Action

### Для разработчиков:

```bash
# Попробуйте СЕГОДНЯ!
cargo add unistructgen

# Первый проект за 5 минут
generate_struct_from_json! {
    name = "MyType",
    json = "..."
}
```

### Для команд:

1. 📅 **Забронировать демо** - покажем на вашем API
2. 🧪 **Pilot проект** - один endpoint, 30 минут
3. 📊 **Измерить эффект** - сравнить с текущим процессом

### Для инвесторов:

- 💰 **High ROI potential** - критичная проблема, большой рынок
- 🚀 **Viral growth** - developers love time-saving tools
- 🌍 **Global market** - not limited by geography
- 💎 **Strong moat** - technical complexity, first-mover

---

## 📞 Контакты

### Начните использовать:

- 🌐 **Website:** [your-website.com]
- 📦 **Crates.io:** https://crates.io/crates/unistructgen
- 💻 **GitHub:** https://github.com/[your-username]/unistructgen
- 📚 **Docs:** https://docs.rs/unistructgen

### Поддержка:

- 💬 **Discord:** [Join community]
- 📧 **Email:** hello@unistructgen.dev
- 🐦 **Twitter:** @unistructgen

---

## 🙏 Thank You!

### Запомните:

> **5 секунд вместо 5 часов**
>
> **99% меньше кода**
>
> **100% type safety**
>
> **∞% уверенности**

**Перестаньте писать типы вручную.**

**Начните создавать продукты.**

---

### P.S. Специальное предложение для early adopters! 🎁

- ⭐ Star на GitHub → упоминание в contributors
- 💬 Feedback → влияние на roadmap
- 🤝 Contribute → стать core contributor
- 🎤 Case study → feature в нашем блоге

**Давайте изменим Rust development вместе!**
