-- This file should undo anything in `up.sql`
alter table comment_upvotes
    drop column type;

alter table video_upvotes
    drop column type;