-- Add up migration script here
create table chat_bot
(
    id SERIAL PRIMARY KEY,
    oauth_token text not null,
    refresh_token text not null,
    owner varchar(330) not null unique
);