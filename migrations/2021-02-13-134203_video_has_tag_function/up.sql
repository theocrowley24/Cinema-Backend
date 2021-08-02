-- Your SQL goes here
drop function if exists video_has_tag(v_id int, t_id int);
create function video_has_tag(v_id int, t_id int)
    returns boolean
    language plpgsql
as
$$
begin
    return exists (select * from videos_tags where video_id = v_id and tag_id = t_id);
end;
$$;