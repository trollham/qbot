-- Add up migration script here
create table queues (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL,
    owner INTEGER NOT NULL REFERENCES users,
    content JSON NOT NULL
);