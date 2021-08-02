-- This file should undo anything in `up.sql`
alter table videos_tags
    drop constraint fk_video;

alter table videos_tags
    drop constraint fk_tag;