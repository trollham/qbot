-- Add up migration script here
alter table queue drop column owner,
    add column owner integer not null,
    add constraint ownerfk foreign key (owner) references users;