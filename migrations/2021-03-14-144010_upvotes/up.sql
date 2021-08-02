-- Your SQL goes here
create table comment_upvotes
(
    id serial not null primary key ,
    user_id integer not null,
    comment_id integer not null,
    inactive bool not null default false,
    date timestamp default CURRENT_TIMESTAMP not null
);

alter table comment_upvotes drop constraint if exists fk_user;
alter table comment_upvotes
    add constraint fk_user
        foreign key (user_id)
            references users (id)
            on delete set null;

alter table comment_upvotes drop constraint if exists fk_comment;
alter table comment_upvotes
    add constraint fk_comment
        foreign key (comment_id)
            references comments (id)
            on delete set null;

create table video_upvotes
(
    id serial not null primary key ,
    user_id integer not null,
    video_id integer not null,
    inactive bool not null default false,
    date timestamp default CURRENT_TIMESTAMP not null
);

alter table video_upvotes drop constraint if exists fk_user;
alter table video_upvotes
    add constraint fk_user
        foreign key (user_id)
            references users (id)
            on delete set null;

alter table video_upvotes drop constraint if exists fk_video;
alter table video_upvotes
    add constraint fk_video
        foreign key (video_id)
            references videos (id)
            on delete set null;