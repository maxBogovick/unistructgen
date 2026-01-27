use anyhow::Result;
use serde::{Deserialize, Serialize};

// Ручная модель под JSON с разными стилями полей
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserProfile {
    #[serde(rename = "userId")]
    user_id: i64,

    #[serde(rename = "fullName")]
    full_name: String,

    #[serde(rename = "isActive")]
    is_active: bool,

    #[serde(rename = "role")]
    role: Option<String>,
}

fn main() -> Result<()> {
    let raw = r#"{
        \"userId\": 1001,
        \"fullName\": \"Alice Johnson\",
        \"isActive\": true,
        \"role\": \"admin\"
    }"#;

    let profile: UserProfile = serde_json::from_str(raw)?;
    println!("Manual profile: {:?}", profile);

    Ok(())
}
