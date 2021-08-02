-- Your SQL goes here
alter table users
    add column if not exists stripe_customer varchar(32) not null default '';

alter table users
    add column if not exists subscribed boolean not null default false;