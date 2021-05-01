-- Add up migration script here
create table queues (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL,
    owner integer REFERENCES users,
    content JSON
);