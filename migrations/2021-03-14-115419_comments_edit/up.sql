-- Your SQL goes here
alter table comments
    add column video_id integer not null;

alter table comments
    add constraint fk_video
        foreign key (video_id)
            references videos
            on delete set null;