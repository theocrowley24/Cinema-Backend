-- Your SQL goes here
alter table users
    add column subscriptions_enabled boolean not null default false,
    add column display_name varchar(128),
    add column bio varchar(1024);
