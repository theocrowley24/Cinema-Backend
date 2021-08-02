-- Your SQL goes here
alter table comment_upvotes
    add column type varchar(16) not null;

alter table video_upvotes
    add column type varchar(16) not null;