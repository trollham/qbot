-- Add down migration script here
alter table queue drop column owner,
    add column owner varchar(330) not null;