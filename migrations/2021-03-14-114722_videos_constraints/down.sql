-- This file should undo anything in `up.sql`
alter table videos
    drop constraint fk_user;