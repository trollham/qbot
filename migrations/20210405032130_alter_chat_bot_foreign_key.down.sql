-- Add down migration script here
alter table chat_bot drop column owner,
    add column owner varchar(330) not null unique;