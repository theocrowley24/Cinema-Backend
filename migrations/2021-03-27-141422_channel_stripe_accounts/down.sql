-- This file should undo anything in `up.sql`
alter table users
    drop column stripe_account;

alter table users
    drop column channel_onboarded;