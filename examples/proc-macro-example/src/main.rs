use unistructgen_macro::{generate_struct_from_json, json_struct};

// Example 1: Using the function-like macro
generate_struct_from_json! {
    name = "Person",
    json = r#"{
        "id": 1,
        "name": "Alice Johnson",
        "email": "alice@example.com",
        "age": 30,
        "is_active": true
    }"#
}

// Example 2: Using the attribute macro
#[json_struct(name = "Product")]
const PRODUCT_SAMPLE: &str = r#"{
    "product_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Laptop",
    "price": 999.99,
    "in_stock": true,
    "tags": ["electronics", "computers"]
}"#;

// Example 3: Nested structures
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

// Example 4: Optional fields
generate_struct_from_json! {
    name = "Config",
    optional = true,
    json = r#"{
        "host": "localhost",
        "port": 8080,
        "debug": true
    }"#
}

fn main() {
    println!("=== Example 1: Person ===");
    let person_json = r#"{
        "id": 42,
        "name": "Bob Smith",
        "email": "bob@example.com",
        "age": 25,
        "is_active": false
    }"#;

    let person: Person = serde_json::from_str(person_json).unwrap();
    println!("Parsed person: {:?}", person);
    println!("Name: {}", person.name);
    println!("Age: {}", person.age);
    println!();

    println!("=== Example 2: Product ===");
    let product: Product = serde_json::from_str(PRODUCT_SAMPLE).unwrap();
    println!("Parsed product: {:?}", product);
    println!("Product name: {}", product.name);
    println!("Price: ${}", product.price);
    println!("Tags: {:?}", product.tags);
    println!();

    println!("=== Example 3: Company (with nested structs) ===");
    let company_json = r#"{
        "name": "Tech Startup",
        "founded": 2024,
        "address": {
            "street": "456 Innovation Blvd",
            "city": "San Francisco",
            "zip": "94102"
        },
        "employees": [
            {
                "name": "Alice Cooper",
                "position": "CEO"
            },
            {
                "name": "Charlie Brown",
                "position": "CTO"
            }
        ]
    }"#;

    let company: Company = serde_json::from_str(company_json).unwrap();
    println!("Parsed company: {:?}", company);
    println!("Company: {}", company.name);
    println!("Location: {}, {}", company.address.city, company.address.street);
    println!("Employees: {}", company.employees.len());
    for emp in &company.employees {
        println!("  - {}: {}", emp.name, emp.position);
    }
    println!();

    println!("=== Example 4: Config (optional fields) ===");
    let config_json = r#"{"host": "example.com"}"#;
    let config: Config = serde_json::from_str(config_json).unwrap();
    println!("Parsed config: {:?}", config);
    println!("Host: {:?}", config.host);
    println!("Port: {:?}", config.port); // Will be None
    println!("Debug: {:?}", config.debug); // Will be None
    println!();

    println!("=== Serialization test ===");
    let new_person = Person {
        id: 100,
        name: "Eve Adams".to_string(),
        email: "eve@example.com".to_string(),
        age: 28,
        is_active: true,
    };

    let serialized = serde_json::to_string_pretty(&new_person).unwrap();
    println!("Serialized person:\n{}", serialized);
}
