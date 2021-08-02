-- Your SQL goes here
create or replace function user_is_subscribed(u_id integer, c_id integer)
    returns boolean
    language 'plpgsql'
as $BODY$
begin
    return exists(
            select *
            from channels_tokens
                     left join tokens on "tokens"."id" = "channels_tokens"."token_id"
            where "tokens"."user_id" = u_id and "channels_tokens"."channel_user_id" = c_id and "channels_tokens"."expires" > current_timestamp
        );
end
$BODY$;