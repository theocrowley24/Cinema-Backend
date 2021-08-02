-- Your SQL goes here
create table if not exists tokens
(
    id serial not null primary key ,
    user_id integer not null,
    used boolean default false not null,
    date_granted timestamp default CURRENT_TIMESTAMP not null,
    date_used timestamp
);

alter table tokens drop constraint if exists fk_user;
alter table tokens
    add constraint fk_user
        foreign key (user_id)
            references users (id)
            on delete set null;