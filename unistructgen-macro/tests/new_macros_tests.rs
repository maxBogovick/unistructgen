use unistructgen_macro::{generate_struct_from_env, generate_struct_from_sql, generate_struct_from_graphql};
use serde;

// Test Env
generate_struct_from_env! {
    name = "AppConfig",
    env = r#"
        # Database
        DB_HOST=localhost
        DB_PORT=5432
        
        # App
        DEBUG=true
        API_KEY="secret_key"
        TIMEOUT=30.5
    "#,
    serde = true
}

// Test SQL
generate_struct_from_sql! {
    sql = r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY NOT NULL,
            username VARCHAR(50) NOT NULL,
            email VARCHAR(255) NOT NULL,
            active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMP
        );

        CREATE TABLE products (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            price DECIMAL(10, 2)
        );
    "#,
    serde = true
}

// Test GraphQL
generate_struct_from_graphql! {
    schema = r#"
        type Customer {
            id: ID!
            name: String!
            age: Int
            email: String
            isActive: Boolean!
        }

        type Order {
            id: ID!
            amount: Float!
            customer: Customer
        }
    "#,
    serde = true
}

#[test]
fn test_env_macro() {
    let config = AppConfig {
        db_host: "localhost".to_string(),
        db_port: 5432,
        debug: true,
        api_key: "secret_key".to_string(),
        timeout: 30.5,
    };
    
    assert_eq!(config.db_host, "localhost");
    assert_eq!(config.db_port, 5432);
    assert!(config.debug);
}

#[test]
fn test_sql_macro() {
    let user = Users {
        id: 1,
        username: "user1".to_string(),
        email: "user@example.com".to_string(),
        active: Some(true),
        created_at: None,
    };

    assert_eq!(user.username, "user1");

    let product = Products {
        id: 1,
        name: "Widget".to_string(),
        price: Some(19.99),
    };
    
    assert_eq!(product.name, "Widget");
}

#[test]
fn test_graphql_macro() {
    let customer = Customer {
        id: "cust_1".to_string(),
        name: "Alice".to_string(),
        age: Some(30),
        email: Some("alice@example.com".to_string()),
        is_active: true,
    };

    assert_eq!(customer.name, "Alice");
    
    let order = Order {
        id: "ord_1".to_string(),
        amount: 99.99,
        customer: Some(customer),
    };
    
    assert_eq!(order.amount, 99.99);
}
