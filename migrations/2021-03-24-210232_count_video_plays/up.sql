-- Your SQL goes here
create or replace function count_video_plays(v_id integer, out result integer)
    returns integer
    language 'plpgsql'
as $BODY$
begin
    select count(*) into result
    from video_plays
    where video_id = v_id;

    return;
end
$BODY$;