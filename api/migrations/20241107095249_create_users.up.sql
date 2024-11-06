-- Add up migration script here

CREATE TABLE users (
    id TEXT NOT NULL PRIMARY KEY,
    settings_json TEXT NOT NULL,
    last_run_at TIMESTAMP 
);
