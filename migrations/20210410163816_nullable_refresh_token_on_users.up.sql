-- Add up migration script here
alter table users alter column refresh_token drop not null;