-- Add down migration script here
alter table users alter column refresh_token set not null;
