use unistructgen_macro::{generate_struct_from_json, json_struct, openapi_to_rust, struct_from_external_api};

#[allow(dead_code)]
#[json_struct(name = "Product")]
const PRODUCT_SAMPLE: &str = r#"{
    "product_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Laptop",
    "price": 999.99,
    "in_stock": true,
    "tags": ["electronics", "computers"]
}"#;

generate_struct_from_json! {
    name = "Company",
    json = r#"{
        "name": "Acme Corp",
        "founded": 2020,
        "address": {
            "street": "123 Main St",
            "city": "New York",
            "zip": "10001"
        },
        "employees": [
            {
                "name": "John Doe",
                "position": "Engineer"
            }
        ]
    }"#
}
generate_struct_from_json! {
    name = "Companys",
    json = r#"{
        "name": "Acme Corp",
        "founded": 2020,
        "address2": {
            "street": "123 Main St",
            "city": "New York",
            "zip": "10001"
        },
        "employees2": [
            {
                "name": "John Doe",
                "position": "Engineer"
            }
        ]
    }"#
}

struct_from_external_api! {
    struct_name = "Exm",
    url_api = "https://jsonplaceholder.typicode.com/todos"
}

struct_from_external_api! {
    struct_name = "ExmAuthEnv",
    url_api = "https://jsonplaceholder.typicode.com/todos/1",
    auth_bearer_env = "API_TOKEN",
    env_file = "tests/fixtures/auth.env"
}

#[test]
fn test_json_struct_attribute() {
    let product = Product {
        product_id: uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        name: "Laptop".to_string(),
        price: 999.99,
        in_stock: true,
        tags: vec!["electronics".to_string(), "computers".to_string()],
    };
    assert_eq!(product.name, "Laptop");
    assert_eq!(product.price, 999.99);
}

#[test]
fn it_works() {
    let company = Company {
        address: Address {
            city: "Odessa".to_string(),
            street: "Malinovskogo".to_string(),
            zip: "65074".to_string()
        },
        employees: vec![],
        founded: 3035,
        name: "Acme Corp".to_string(),
    };
    assert_eq!(company.address.city, "Odessa".to_string());
}

#[test]
fn test_array_api_response() {
    // Test that struct_from_external_api works with API that returns an array
    // The macro should automatically extract the first element
    let todo = Exm {
        user_id: 1,
        id: 1,
        title: "Test todo".to_string(),
        completed: false,
    };
    assert_eq!(todo.user_id, 1);
    assert_eq!(todo.completed, false);
}

#[test]
fn test_external_api_env_auth() {
    // Ensure struct_from_external_api reads bearer token from env_file
    let todo = ExmAuthEnv {
        user_id: 1,
        id: 1,
        title: "delectus aut autem".to_string(),
        completed: false,
    };
    assert_eq!(todo.id, 1);
    assert_eq!(todo.completed, false);
}

// Test openapi_to_rust! macro with inline spec
openapi_to_rust! {
    spec = r#"
openapi: 3.0.0
info:
  title: Simple API
  version: 1.0.0
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
components:
  schemas:
    TestUser:
      type: object
      required:
        - id
        - username
      properties:
        id:
          type: integer
        username:
          type: string
        email:
          type: string
"#
}

#[test]
fn test_openapi_inline_spec() {
    // Test that openapi_to_rust! generates structs from inline spec
    let user = TestUser {
        id: 1,
        username: "alice".to_string(),
        email: Some("alice@example.com".to_string()),
    };
    assert_eq!(user.id, 1);
    assert_eq!(user.username, "alice");
    assert_eq!(user.email, Some("alice@example.com".to_string()));
}

// Test openapi_to_rust! macro with file
openapi_to_rust! {
    file = "tests/test-api.yaml"
}

#[test]
fn test_openapi_from_file() {
    // Test that openapi_to_rust! generates structs from file
    let user = User {
        id: 123,
        name: "Bob".to_string(),
        email: Some("bob@test.com".to_string()),
        age: Some(30),
    };
    assert_eq!(user.id, 123);
    assert_eq!(user.name, "Bob");

    let post = Post {
        id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        title: "Hello World".to_string(),
        content: "This is a test post".to_string(),
        published: Some(true),
    };
    assert_eq!(post.title, "Hello World");
}

#[test]
fn test_openapi_validation() {
    use validator::Validate;

    // Test valid User
    let valid_user = User {
        id: 1,
        name: "A".to_string(), // minLength: 1 - should be valid
        email: Some("test@example.com".to_string()),
        age: Some(25),
    };
    assert!(valid_user.validate().is_ok(), "Valid user should pass validation");

    // Test Post validation
    let valid_post = Post {
        id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        title: "Hello".to_string(), // minLength: 5 - should be valid
        content: "This is content".to_string(),
        published: Some(false),
    };
    assert!(valid_post.validate().is_ok(), "Valid post should pass validation");

    // Test validation with short title (should fail)
    let invalid_post = Post {
        id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        title: "Hi".to_string(), // minLength: 5 - should FAIL
        content: "Content".to_string(),
        published: Some(false),
    };
    assert!(invalid_post.validate().is_err(), "Post with short title should fail validation");
}

#[test]
fn test_openapi_optional_fields() {
    // Test that optional fields work correctly
    let user_without_email = User {
        id: 2,
        name: "Charlie".to_string(),
        email: None, // optional field
        age: None,   // optional field
    };
    assert_eq!(user_without_email.email, None);
    assert_eq!(user_without_email.age, None);

    // Test Post with default value
    let post_without_published = Post {
        id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
        title: "Test Post".to_string(),
        content: "Content".to_string(),
        published: None, // Has default: false
    };
    // When published is None, it's treated as None (not automatically set to false by our generator)
    // The OpenAPI default is more of a documentation hint
    assert_eq!(post_without_published.published, None);
}

#[test]
fn test_openapi_generated_types_are_serializable() {
    use serde_json;

    let user = User {
        id: 100,
        name: "David".to_string(),
        email: Some("david@test.com".to_string()),
        age: Some(35),
    };

    // Should be able to serialize to JSON
    let json = serde_json::to_string(&user).expect("Should serialize to JSON");
    assert!(json.contains("David"));
    assert!(json.contains("david@test.com"));

    // Should be able to deserialize from JSON
    let json_str = r#"{"id":200,"name":"Eve","email":"eve@test.com","age":28}"#;
    let deserialized: User = serde_json::from_str(json_str).expect("Should deserialize from JSON");
    assert_eq!(deserialized.id, 200);
    assert_eq!(deserialized.name, "Eve");
    assert_eq!(deserialized.email, Some("eve@test.com".to_string()));
    assert_eq!(deserialized.age, Some(28));
}
