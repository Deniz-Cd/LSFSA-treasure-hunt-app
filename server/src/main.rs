use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ─── Data Model ────────────────────────────────────────────────────────────────
//
// This struct represents one row in our CSV file.
// `Serialize` lets us convert it TO JSON (for responses).
// `Deserialize` lets us read it FROM the CSV.
//
#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: u32,
    name: String,
    age: u32,
    city: String,
}

// ─── Shared State ──────────────────────────────────────────────────────────────
//
// We load the CSV once at startup and store it in memory.
// `Arc` (Atomic Reference Counted) lets us safely share this
// data across multiple requests without copying it.
//
struct AppState {
    users: Vec<User>,
}

// ─── CSV Loader ────────────────────────────────────────────────────────────────
fn load_users_from_csv(path: &str) -> Result<Vec<User>, csv::Error> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut users = Vec::new();

    // `deserialize()` uses serde under the hood to map CSV columns
    // to our User struct fields automatically — no manual parsing!
    for result in reader.deserialize() {
        let user: User = result?;
        users.push(user);
    }

    Ok(users)
}

// ─── Handlers ──────────────────────────────────────────────────────────────────
//
// Handlers are async functions that axum calls when a route is matched.
// They receive extractors (like State, Path) and return a response.

// GET /users  →  returns all users as a JSON array
async fn get_all_users(State(state): State<Arc<AppState>>) -> Json<Vec<User>> {
    // `Json(...)` wraps our data and sets Content-Type: application/json
    Json(state.users.clone())
}

// GET /users/:id  →  returns one user or 404
async fn get_user_by_id(
    Path(id): Path<u32>,           // axum extracts :id from the URL path
    State(state): State<Arc<AppState>>,
) -> Result<Json<User>, StatusCode> {
    match state.users.iter().find(|u| u.id == id) {
        Some(user) => Ok(Json(user.clone())),
        None => Err(StatusCode::NOT_FOUND),   // returns HTTP 404
    }
}

// GET /users/city/:city  →  returns users matching a city
async fn get_users_by_city(
    Path(city): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<Vec<User>> {
    let filtered: Vec<User> = state
        .users
        .iter()
        .filter(|u| u.city.to_lowercase() == city.to_lowercase())
        .cloned()
        .collect();

    Json(filtered)
}

// ─── Main ──────────────────────────────────────────────────────────────────────
#[tokio::main]
async fn main() {
    // Load the CSV — panic early if it's missing or malformed
    let users = load_users_from_csv("users.csv").expect("Failed to load users.csv");
    println!("Loaded {} users from CSV", users.len());

    // Wrap state in Arc so axum can clone it cheaply for each request
    let shared_state = Arc::new(AppState { users });

    // Define routes — think of this as a URL map
    let app = Router::new()
        .route("/users", get(get_all_users))
        .route("/users/:id", get(get_user_by_id))
        .route("/users/city/:city", get(get_users_by_city))
        .with_state(shared_state);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Server running on http://{}", addr);

    // `serve` drives the async event loop — it never returns unless it errors
    axum::serve(listener, app).await.unwrap();
}
