-- Your SQL goes here
-- Your SQL goes here
create or replace function video_recently_watched(u_id integer, v_id integer)
    returns boolean
    language 'plpgsql'
as $BODY$
begin
    return exists(
            select *
            from video_plays
            where user_id = u_id and video_id = v_id and date < current_timestamp + interval '5 days'
        );
end
$BODY$;