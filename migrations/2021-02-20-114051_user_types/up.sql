-- Your SQL goes here
alter table users
    add column if not exists user_type varchar(16) not null default 'SUBSCRIBER';