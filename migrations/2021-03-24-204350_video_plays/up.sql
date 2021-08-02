-- Your SQL goes here
create table video_plays (
    id serial not null primary key ,
    user_id integer not null ,
    video_id integer not null ,
    date timestamp default CURRENT_TIMESTAMP not null
);

alter table video_plays drop constraint if exists fk_user;
alter table video_plays
    add constraint fk_user
        foreign key (user_id)
            references users (id)
            on delete set null;

alter table video_plays drop constraint if exists fk_video;
alter table video_plays
    add constraint fk_video
        foreign key (video_id)
            references videos (id)
            on delete set null;