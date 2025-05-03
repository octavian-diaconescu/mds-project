use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password_hash: String,
}   
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthenticatedUser {
    pub id: i32,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDevice {
    pub user_id: i32,
    pub thing_name: String,
}

// For registration/login requests
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
}

// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub exp: usize,  // Expiration timestamp
    pub is_admin: bool,
}