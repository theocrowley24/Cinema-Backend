-- Your SQL goes here
alter table comment_upvotes
rename column type to upvote_type;

alter table video_upvotes
    rename column type to upvote_type;