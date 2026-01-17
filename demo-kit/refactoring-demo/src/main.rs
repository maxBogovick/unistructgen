// ❌ THE "BEFORE" STATE: Manual Boilerplate Hell
// Imagine maintaining this file manually...

use serde::{Deserialize, Serialize};

// --- START OF BOILERPLATE ---
// We have to define 4 structs just to fetch one user!
// If the API adds a field, we miss it.
// If types change, we crash.

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub email: String,
    pub address: Address, // Nested struct
    pub phone: String,
    pub website: String,
    pub company: Company, // Another nested struct
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub suite: String,
    pub city: String,
    pub zipcode: String,
    pub geo: Geo, // Even deeper nesting!
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Geo {
    pub lat: String,
    pub lng: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Company {
    pub name: String,
    pub catch_phrase: Option<String>, // Maybe optional? Guessing...
    pub bs: Option<String>,
}
// --- END OF BOILERPLATE ---

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Fetching User (Manual Mode) ---");

    // We manually fetch data...
    let url = "https://jsonplaceholder.typicode.com/users/1";
    let resp = reqwest::blocking::get(url)?.text()?;
    
    // And try to map it to our manually written structs
    let user: User = serde_json::from_str(&resp)?;

    println!("User: {} ({})", user.name, user.email);
    println!("City: {}", user.address.city);
    println!("Company: {}", user.company.name);

    Ok(())
}
