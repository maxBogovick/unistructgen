use anyhow::Result;
use unistructgen_macro::generate_struct_from_json;

// UniStructGen сам генерирует структуру и serde‑rename
// + можно делать все поля optional, если входные данные нестабильны

generate_struct_from_json! {
    name = "UserProfile",
    json = r#"{
        \"userId\": 1001,
        \"fullName\": \"Alice Johnson\",
        \"isActive\": true,
        \"role\": \"admin\"
    }"#,
    serde = true,
    optional = false
}

fn main() -> Result<()> {
    let profile = UserProfile {
        user_id: 1001,
        full_name: "Alice Johnson".to_string(),
        is_active: true,
        role: Some("admin".to_string()),
    };

    println!("Generated profile: {:?}", profile);
    Ok(())
}
