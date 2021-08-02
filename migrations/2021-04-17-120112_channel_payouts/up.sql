-- Your SQL goes here
alter table channels_tokens
    add column converted boolean not null default false;

create table token_transactions(
    id serial not null primary key ,
    channel_user_id integer not null ,
    transaction_type varchar(128) not null ,
    amount integer not null ,
    date timestamp default CURRENT_TIMESTAMP not null
)