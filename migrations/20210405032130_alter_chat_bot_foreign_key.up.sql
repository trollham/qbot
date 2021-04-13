-- Add up migration script here
alter table chat_bot drop column owner,
    add column owner integer not null,
    add constraint ownerfk foreign key (owner) references users