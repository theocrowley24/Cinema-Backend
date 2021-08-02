-- This file should undo anything in `up.sql`
alter table comment_upvotes
    rename column upvote_type to type;

alter table video_upvotes
    rename column upvote_type to type;