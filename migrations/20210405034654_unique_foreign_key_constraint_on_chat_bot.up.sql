-- Add up migration script here
alter table chat_bot
add unique (owner);