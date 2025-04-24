use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use chrono::Utc;
use crate::models::Claims;

const JWT_EXPIRATION_HOURS: i64 = 12;

pub fn create_jwt(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(JWT_EXPIRATION_HOURS))
        .expect("Invalid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        user_id,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()),
    )
}

pub fn validate_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}