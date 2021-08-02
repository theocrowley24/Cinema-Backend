-- This file should undo anything in `up.sql`
alter table users
    drop column avatar_filename,
    drop column cover_filename