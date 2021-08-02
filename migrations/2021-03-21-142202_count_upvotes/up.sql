-- Your SQL goes here
create or replace function count_comment_upvotes(c_id integer, out result integer)
returns integer
language 'plpgsql'
as $BODY$
    begin
        select count(*) into result
        from comment_upvotes
        where comment_id = c_id
          and inactive = false
          and upvote_type ='UP';

        return;
    end
$BODY$;

create or replace function count_comment_downvotes(c_id integer, out result integer)
returns integer
language 'plpgsql'
as $BODY$
begin
    select count(*) into result
    from comment_upvotes
    where comment_id = c_id
      and inactive = false
      and upvote_type ='DOWN';

    return;
end
$BODY$;

create or replace function count_video_upvotes(v_id integer, out result integer)
returns integer
language 'plpgsql'
as $BODY$
begin
    select count(*) into result
    from video_upvotes
    where video_id = v_id
      and inactive = false
      and upvote_type ='UP';

    return;
end
$BODY$;

create or replace function count_video_downvotes(v_id integer, out result integer)
returns integer
language 'plpgsql'
as $BODY$
begin
    select count(*) into result
    from video_upvotes
    where video_id = v_id
      and inactive = false
      and upvote_type ='DOWN';

    return;
end
$BODY$;