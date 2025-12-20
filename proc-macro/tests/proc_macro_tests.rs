use unistructgen_macro::{generate_struct_from_json, json_struct, struct_from_external_api};

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
    url_api = "https://jsonplaceholder.typicode.com/todos/1"
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