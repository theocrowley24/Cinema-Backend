-- Your SQL goes here
alter table videos drop constraint if exists fk_user;
alter table videos
    add constraint fk_user
        foreign key (user_id)
            references users (id)
            on delete set null;