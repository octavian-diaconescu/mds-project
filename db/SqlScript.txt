CREATE DATABASE iot_security;
\c iot_security

CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE device_metrics (
    time TIMESTAMPTZ NOT NULL,
    device_id TEXT NOT NULL,
    metric_type TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL
);

SELECT create_hypertable('device_metrics', 'time');

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    role TEXT DEFAULT 'user'
);

CREATE TABLE user_devices (
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    thing_name TEXT NOT NULL,
    PRIMARY KEY (user_id, thing_name)
);