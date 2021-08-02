-- Your SQL goes here
create table if not exists channels_tokens
(
    id serial not null primary key ,
    token_id integer not null,
    channel_user_id integer not null
);

alter table channels_tokens drop constraint if exists fk_token;
alter table channels_tokens
    add constraint fk_token
        foreign key (token_id)
            references tokens (id)
            on delete set null;

alter table channels_tokens drop constraint if exists fk_user;
alter table channels_tokens
    add constraint fk_user
        foreign key (channel_user_id)
            references users (id)
            on delete set null;