use axum::{
    http::StatusCode,
    extract::{Path, State},
    routing::{get, post, delete},
    Json,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::vec::Vec;

// We'll create a unique ID for each user.
static mut NEXT_ID: u32 = 0;

// The struct for incoming JSON data. We add an 'id' field to uniquely identify users
#[derive(Deserialize, Serialize, Debug, Clone)]
struct UserData {
    // The ID field is optional for the incoming JSON from POST requests
    // It will be assigned by the server.
    #[serde(default)]
    id: u32,
    name: String,
    age: u32,
}

// The application's state to hold our in-memory database
struct AppState {
    users: Arc<Mutex<Vec<UserData>>>,
}

//---

// Main function to run the application.
#[tokio::main]
async fn main() {
    // Initialize the application state with an empty vector.
    let app_state = Arc::new(Mutex::new(Vec::new()));

    // Define the application routes and attach the shared state.
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/greet/:name", get(greet_person))
        .route("/users", post(add_user).get(list_users))
        .route("/users/:id", delete(delete_user))
        .with_state(app_state);

    // Set up the server address.
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running on http://{}", addr);

    // Start the server.
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}



// Handler for the root route
async fn hello_world() -> &'static str {
    "Hello, world!"
}

// Handler to greet a person based on a URL parameter.
async fn greet_person(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

// Handler to add a new user to our in-memory database.
async fn add_user(
    State(users_state): State<Arc<Mutex<Vec<UserData>>>>,
    Json(mut payload): Json<UserData>,
) -> Json<UserData> {
    let mut users = users_state.lock().unwrap();
    
    // Assign a unique ID to the new user.
    unsafe {
        payload.id = NEXT_ID;
        NEXT_ID += 1;
    }

    // Add the new user to the list.
    users.push(payload.clone());

    println!("New user added. Total users: {}", users.len());
    
    Json(payload)
}

// Handler to list all users as JSON.
async fn list_users(
    State(users_state): State<Arc<Mutex<Vec<UserData>>>>,
) -> Json<Vec<UserData>> {
    let users = users_state.lock().unwrap();
    
    Json(users.clone())
}

// New handler for deleting a user by ID with professional error handling
async fn delete_user(
    State(users_state): State<Arc<Mutex<Vec<UserData>>>>,
    Path(id): Path<u32>,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let mut users = users_state.lock().unwrap();

    let initial_len = users.len();
    users.retain(|user| user.id != id);

    // Check if the user was actually removed.
    if users.len() < initial_len {
        Ok(format!("User with ID {} successfully deleted.", id))
    } else {
        // If not found, return a 404 Not Found status with a JSON error.
        let error_message = format!("User with ID {} not found.", id);
        let json_error = serde_json::json!({ "error": error_message });
        Err((StatusCode::NOT_FOUND, Json(json_error)))
    }
}