-- Add up migration script here
create table queue (
    id SERIAL PRIMARY KEY,
    name varchar(80) not null,
    owner varchar(330) not null,
    content json
);