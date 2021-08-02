-- This file should undo anything in `up.sql`
alter table users
    drop column subscriptions_enabled,
    drop column display_name,
    drop column bio;