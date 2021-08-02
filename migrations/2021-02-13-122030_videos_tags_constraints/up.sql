-- Your SQL goes here
alter table videos_tags drop constraint if exists fk_video;
alter table videos_tags
    add constraint fk_video
        foreign key (video_id)
            references videos (id)
            on delete set null;

alter table videos_tags drop constraint if exists fk_tag;
alter table videos_tags
    add constraint fk_tag
        foreign key (tag_id)
            references tags (id)
            on delete set null;