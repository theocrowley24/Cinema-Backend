-- This file should undo anything in `up.sql`
alter table channels_tokens
    drop column converted;

drop table token_transactions;