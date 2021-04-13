-- Add down migration script here
alter table users drop constraint users_email_key;
