-- Add up migration script here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email varchar(330) UNIQUE NOT NULL,
    refresh_token TEXT,
    username TEXT NOT NULL
)