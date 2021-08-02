-- Your SQL goes here
create table if not exists comments
(
    id serial not null primary key ,
    user_id integer not null,
    text varchar(256) not null,
    inactive bool not null default false,
    date timestamp default CURRENT_TIMESTAMP not null
);

alter table comments drop constraint if exists fk_user;
alter table comments
    add constraint fk_user
        foreign key (user_id)
            references users (id)
            on delete set null;