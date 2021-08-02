create table if not exists tags
(
    id serial not null primary key,
    name varchar(128) not null
);

create table if not exists users
(
    id serial not null
        constraint users_pkey
            primary key,
    username varchar(128) not null
        constraint users_username_key
            unique,
    password varchar(256) not null,
    email varchar(256) not null
        constraint users_email_key
            unique,
    password_reset_token varchar(256)
);

create table if not exists videos
(
    id serial not null primary key ,
    file_name varchar(128) not null,
    user_id integer not null,
    title varchar(256) not null,
    description varchar(1024),
    upload_date timestamp default CURRENT_TIMESTAMP not null,
    status varchar(256) default 'WAITING' not null
);

create table if not exists videos_tags
(
    id serial not null primary key ,
    video_id integer not null,
    tag_id integer not null
);

