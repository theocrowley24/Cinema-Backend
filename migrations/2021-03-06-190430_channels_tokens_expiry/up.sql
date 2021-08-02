-- Your SQL goes here
alter table channels_tokens
    add column if not exists expires timestamp not null default now() + interval '30 days';