-- Your SQL goes here
alter table users
    add column stripe_account varchar(32);

alter table users
    add column channel_onboarded boolean not null default false;