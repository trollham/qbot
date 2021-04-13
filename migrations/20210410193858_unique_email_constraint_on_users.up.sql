-- Add up migration script here
alter table users
add unique (email);