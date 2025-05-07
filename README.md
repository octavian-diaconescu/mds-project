# IoT Security Platform

A secure IoT device management platform with real-time temperature monitoring and anomaly detection.

## Features

- **User Authentication & Authorization**
  - JWT-based authentication
  - Role-based access control (Admin/User)
  - Secure password hashing

- **Device Management**
  - Create and manage IoT devices
  - Real-time device monitoring
  - Device ownership validation

- **Security Features**
  - AWS IoT Core integration
  - TLS/MQTT secure communication
  - Anomaly detection system

- **Data Visualization**
  - Real-time temperature graphs
  - Anomaly alerts and history
  - Responsive dashboard

## Tech Stack

- **Backend:** Rust, Actix-web, SQLx
- **Frontend:** React, TailwindCSS, Recharts, Axios
- **Database:** PostgreSQL with TimescaleDB, Hypertables for time-series data
- **Cloud:** AWS IoT Core, IAM Roles

## Project Structure
```bash
.
├── iot-security-backend/     # Rust backend service
│   ├── src/
│   │   ├── main.rs          # Main application entry
│   │   ├── auth.rs          # Authentication logic
│   │   ├── middleware.rs    # JWT middleware
│   │   └── models.rs        # Data models
│   └── Cargo.toml
├── web-interface/           # React frontend
│   └── iot-ui/
│       ├── src/
│       │   ├── pages/      # React components
│       │   └── api/        # API client
│       └── package.json
└── db/                     # Database scripts
    └── SqlScript.sql      # Schema definition
```

## Prerequisites

- Rust (stable)
- Node.js (v16+)
- Docker
- PostgreSQL with TimescaleDB
- AWS Account with IoT Core access

## Installation

1. **Clone the repository**
```bash
git clone https://github.com/yourusername/iot-security-platform.git
cd iot-security-platform
```
2. **Set up the backend**
```bash
cd iot-security-backend
```
**Configure your environment variables**
```bash
DATABASE_URL="postgres://postgres:password@localhost:5432/iot_security"
JWT_SECRET="your-secret-key"
AWS_REGION="your-aws-region"
AWS_ACCESS_KEY_ID="your-access-key"
AWS_SECRET_ACCESS_KEY="your-secret-key"
AWS_IOT_ENDPOINT="your-iot-endpoint"
MQTT_CA=your_CA
MQTT_CERT=your_CERT
MQTT_KEY=your_PRIVATE_KEY
```
**Finally, build the backend**
```bash
cargo build
```
3. **Set up the database**
```bash
cd db
psql -U postgres -f SqlScript.SQL
```
4. **Set up the frontend**
```bash
cd web-interface/iot-ui
npm install
```
## Running the Application
1. **Start the database**
2. **Start the backend**
```bash
cd iot-security-backend
cargo run
```
-  Use `cargo clean` to remove artifacts from the target directory. This project takes about 2.5GB of space.
3. **Start the frontend**
```bash
cd web-interface/iot-ui
npm run dev
```
-  You may need to use `sudo npm run dev` on Unix based systems.


