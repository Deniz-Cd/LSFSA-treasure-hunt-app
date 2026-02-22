use serde::{Deserialize, Serialize};

// ─── Data Model ────────────────────────────────────────────────────────────────
//
// The client defines the SAME struct as the server.
// In a real project you'd share this via a common `types` crate.
// `Deserialize` is what we need here — we're reading JSON INTO this struct.
//
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    age: u32,
    city: String,
}

const BASE_URL: &str = "http://127.0.0.1:3000";

// ─── Main ──────────────────────────────────────────────────────────────────────
#[tokio::main]
async fn main() {
    // `reqwest::Client` is reusable — creating one per request is wasteful
    let client = reqwest::Client::new();

    println!("=== Fetching all users ===");
    fetch_all_users(&client).await;

    println!("\n=== Fetching user with id=2 ===");
    fetch_user_by_id(&client, 2).await;

    println!("\n=== Fetching user with id=99 (not found) ===");
    fetch_user_by_id(&client, 99).await;

    println!("\n=== Fetching users in Chicago ===");
    fetch_users_by_city(&client, "Chicago").await;
}

async fn fetch_all_users(client: &reqwest::Client) {
    let url = format!("{}/users", BASE_URL);

    // `.json::<Vec<User>>()` automatically deserializes the JSON response
    // into our Vec<User> — serde handles all the mapping
    match client.get(&url).send().await {
        Ok(response) => match response.json::<Vec<User>>().await {
            Ok(users) => {
                for user in users {
                    println!("  {:?}", user);
                }
            }
            Err(e) => eprintln!("Failed to parse response: {}", e),
        },
        Err(e) => eprintln!("Request failed: {}", e),
    }
}

async fn fetch_user_by_id(client: &reqwest::Client, id: u32) {
    let url = format!("{}/users/{}", BASE_URL, id);

    let response = client.get(&url).send().await.expect("Request failed");

    // Check the HTTP status code before trying to parse the body
    if response.status().is_success() {
        let user: User = response.json().await.expect("Failed to parse user");
        println!("  Found: {:?}", user);
    } else {
        println!("  HTTP {}: user not found", response.status());
    }
}

async fn fetch_users_by_city(client: &reqwest::Client, city: &str) {
    let url = format!("{}/users/city/{}", BASE_URL, city);

    let response = client.get(&url).send().await.expect("Request failed");
    let users: Vec<User> = response.json().await.expect("Failed to parse users");

    if users.is_empty() {
        println!("  No users found in {}", city);
    } else {
        for user in users {
            println!("  {:?}", user);
        }
    }
}
