mod auth;
mod middleware;
mod models;

use crate::models::AuthRequest;
use crate::models::AuthenticatedUser;
use actix_cors::Cors;
// use actix_web::middleware::{from_fn, Logger};
use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use auth::create_jwt;
use aws_sdk_iot::Client;
use bcrypt::{hash, verify, DEFAULT_COST};
use dotenv::dotenv;
use middleware::JwtMiddleware;
// use crate::middleware::jwt_validator;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::collections::HashMap;
use std::env;
// use actix_web_lab::middleware::from_fn;

#[post("/devices")]
async fn create_device(
    payload: web::Json<serde_json::Value>,
    pool: web::Data<PgPool>,
    user: web::ReqData<AuthenticatedUser>,
) -> impl Responder {
    let thing_name = match payload["name"].as_str() {
        Some(name) => name,
        None => return HttpResponse::BadRequest().body("Invalid payload: 'name' is required"),
    };

    // Load AWS configuration from environment
    let config = aws_config::from_env().load().await;
    let iot_client = Client::new(&config);

    // Create the IoT thing with better error handling
    match iot_client
        .create_thing()
        .thing_name(thing_name)
        .send()
        .await
    {
        Ok(_) => {
            // Only insert into database if AWS creation was successful
            match sqlx::query!(
                "INSERT INTO user_devices (user_id, thing_name) VALUES ($1, $2)",
                user.id,
                thing_name,
            )
            .execute(pool.get_ref())
            .await
            {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "thing_name": thing_name,
                    "status": "created"
                })),
                Err(e) => {
                    eprintln!("Database error: {:?}", e);
                    // Attempt to cleanup the AWS thing since DB insert failed
                    let _ = iot_client
                        .delete_thing()
                        .thing_name(thing_name)
                        .send()
                        .await;
                    HttpResponse::InternalServerError().body("Failed to create device")
                }
            }
        }
        Err(e) => {
            eprintln!("AWS error: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create AWS IoT thing",
                "details": format!("{:?}", e)
            }))
        }
    }
}

// #[derive(Serialize)]
// struct DeviceResponse {
//     thing_name: String,
// }
#[derive(Serialize, sqlx::FromRow)]
struct DeviceRecord {
    thing_name: String,
}

#[get("/devices")]
async fn list_devices(pool: web::Data<PgPool>, user: web::ReqData<AuthenticatedUser>) -> impl Responder {
    // let devices = sqlx::query!(
    //     "SELECT thing_name FROM user_devices WHERE user_id = $1",
    //     *user_id
    // )
    // .fetch_all(pool.get_ref())
    // .await
    // .unwrap(); //Break operation if DB error

    let role_result = sqlx::query!(
        "SELECT role FROM users WHERE id = $1",
        user.id
    )
    .fetch_one(pool.get_ref())
    .await;

    let is_admin = match role_result {
        Ok(user) => user.role.unwrap_or("user".to_string()) == "admin",
        Err(_) => false,
    };

    // Fetch all or only user devices
    let devices_result = if is_admin {
        sqlx::query_as::<_, DeviceRecord>("SELECT thing_name FROM user_devices")
            .fetch_all(pool.get_ref())
            .await
    } else {
        sqlx::query_as::<_, DeviceRecord>(
            "SELECT thing_name FROM user_devices WHERE user_id = $1",
        )
        .bind(user.id)
        .fetch_all(pool.get_ref())
        .await
    };

     match devices_result {
        Ok(devices) => HttpResponse::Ok().json(devices),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to fetch devices");
        }
    }
    // let devices: Vec<DeviceResponse> = devices
    //     .into_iter()
    //     .map(|row| DeviceResponse {
    //         thing_name: row.thing_name,
    //     })
    //     .collect();

    // HttpResponse::Ok().json(devices)
}

#[delete("/devices/{thing_name}")]
async fn delete_device(
    path: web::Path<String>, //Extract thing_name from URL as string
    pool: web::Data<PgPool>,
    user: web::ReqData<AuthenticatedUser>,
) -> impl Responder {
    let thing_name = path.into_inner(); //Unwraps into String

    // If not admin, verify ownership
    if !user.is_admin {
        let exists = sqlx::query!(
            "SELECT 1 as exists FROM user_devices WHERE user_id = $1 AND thing_name = $2",
            user.id,
            thing_name
        )
        .fetch_optional(pool.get_ref())
        .await
        .unwrap();

        if exists.is_none() {
            return HttpResponse::Unauthorized().body("Device not owned by user");
        }
    }
    // Delete from AWS IoT Core
    let config = aws_config::load_from_env().await;
    let iot_client = Client::new(&config);

    if let Err(e) = iot_client
        .delete_thing()
        .thing_name(&thing_name)
        .send()
        .await {
            eprintln!("AWS IoT delete error: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to delete thing from AWS IoT");
        }

    // Delete from database
    let _ = sqlx::query!(
        "DELETE FROM user_devices WHERE thing_name = $1",
        thing_name
    )
    .execute(pool.get_ref())
    .await
    .unwrap();

    HttpResponse::Ok().body("Device deleted")
}

#[post("/register")]
async fn register_user(payload: web::Json<AuthRequest>, pool: web::Data<PgPool>) -> impl Responder {
    let hashed_password = hash(&payload.password, DEFAULT_COST).unwrap();

    match sqlx::query!(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id, role",
        payload.email,
        hashed_password
    )
    .fetch_one(pool.get_ref()) //Fetch the inserted row
    .await
    {
        Ok(user) => {
            let is_admin = user.role.as_deref() == Some("admin");
            let token = create_jwt(user.id, is_admin).unwrap();
            HttpResponse::Ok().json(serde_json::json!({ "token": token }))
        }
        Err(e) => HttpResponse::BadRequest().body(format!("Registration failed: {}", e)),
    }
}

#[post("/login")]
async fn login_user(payload: web::Json<AuthRequest>, pool: web::Data<PgPool>) -> impl Responder {
    let user = sqlx::query!(
        "SELECT id, password_hash, role FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(pool.get_ref())
    .await;

    let user = match user {
        Ok(user) => user,
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            }));
        }
    };

    match user {
        Some(user) => {  
            match verify(&payload.password, &user.password_hash) {
                Ok(is_valid) => {
                    if is_valid {
                        let is_admin = user.role.as_deref() == Some("admin");
                        match create_jwt(user.id, is_admin) {
                            Ok(token) => HttpResponse::Ok().json(serde_json::json!({
                                "token": token,
                                "user_id": user.id
                            })),
                            Err(e) => {
                                eprintln!("JWT creation error: {:?}", e);
                                HttpResponse::InternalServerError().json(serde_json::json!({
                                    "error": "Token creation failed"
                                }))
                            }
                        }
                    } else {
                        HttpResponse::Unauthorized().json(serde_json::json!({
                            "error": "Invalid credentials"
                        }))
                    }
                }
                Err(e) => {
                    eprintln!("Password verification error: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Password verification failed"
                    }))
                }
            }
        }
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        })),
    }
}

// struct for incoming JSON data
#[derive(Deserialize, Debug)]
struct SensorData {
    device_id: String,
    temperature: f64,
    timestamp: i64,
}

#[post("/data")]
async fn public_receive_data(data: web::Json<Value>, pool: web::Data<PgPool>) -> impl Responder {
    println!("Received JSON: {}", data);

    let value = data["temperature"].as_f64().unwrap_or(0.0);
    let device_id = data["device_id"].as_str().unwrap_or("unknown_device");

    match sqlx::query!(
        r#"
        INSERT INTO device_metrics (time, device_id, metric_type, value)
        VALUES (NOW(), $1, $2, $3)
        "#,
        device_id,
        "temperature",
        value
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().body("Data stored!"),
        Err(e) => {
            eprintln!("DB error (public /data): {:?}", e);
            HttpResponse::InternalServerError().body("DB error")
        }
    }
}

// HTTP endpoint to receive data
#[post("/data")]
async fn receive_data(data: web::Json<SensorData>, pool: web::Data<PgPool>, user: web::ReqData<AuthenticatedUser>) -> impl Responder {
    // let payload_json = serde_json::json!({
    //     "device_id": data.device_id,
    //     "temperature": data.temperature,
    //     "timestamp": data.timestamp,
    // });
    
    // First verify device ownership
    let _device_exists = sqlx::query!(
        "SELECT 1 AS exists FROM user_devices WHERE user_id = $1 AND thing_name = $2",
        user.id,
        data.device_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to verify device ownership"
        }))
    });

    println!("Received data: {:?}", data);

    if data.temperature > 40.0 {
        println!("ALERT: High temperature detected!");
    }

    // Insert data into TimescaleDB
    match sqlx::query!(
        "INSERT INTO device_metrics (time, device_id, metric_type, value) 
        VALUES (to_timestamp($1), $2, $3, $4)",
        data.timestamp as f64,
        data.device_id,
        "temperature",
        data.temperature
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().body("Data stored!"),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to store data")
        }
    }

}

#[get("/data")]
async fn verify_endpoint(query: web::Query<HashMap<String, String>>) -> impl Responder {
    if let Some(token) = query.get("token") {
        println!("AWS Verification Token: {}", token);
        // return the token as plain text
        HttpResponse::Ok().body(token.to_string())
    } else {
        HttpResponse::BadRequest().body("Token not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .evn variables
    dotenv().ok();
    env_logger::init();

    // Connect to containerized db
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("DATABASE_URL not set in .env");
        std::process::exit(1);
    });
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Database connection error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Start actix server
    println!("Server running at http://localhost:8080");
    HttpServer::new(move || { // Clone DB pool
        App::new()
            .wrap(Cors::permissive()) //Allow any origin
            .app_data(web::Data::new(pool.clone())) //Shared DB pool with all requests 
            .wrap(Logger::default())
            .service(register_user)
            .service(login_user)
            .service(verify_endpoint)
            .service(public_receive_data) //public /data
            .service(
                web::scope("/api")
                .wrap(JwtMiddleware) //Must go through this route
                .service(create_device)
                .service(list_devices)
                .service(delete_device)
                .service(receive_data) //secure /api/data
            )
    })
    .bind("0.0.0.0:8080")? // '?' Propagates errors
    .run()
    .await
}