-- Your SQL goes here
alter table users
    add column avatar_filename varchar(256),
    add column cover_filename varchar(256);