-- Add up migration script here
create table users (
    id serial primary key,
    email varchar(330) not null,
    refresh_token text not null
);