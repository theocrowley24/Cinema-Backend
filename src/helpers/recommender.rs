use crate::establish_connection;
use crate::schema::videos::dsl::{videos, id, status};
use diesel::{QueryDsl, ExpressionMethods, JoinOnDsl, BoolExpressionMethods, QueryResult};
use diesel::dsl::{sql, exists};
use diesel::pg::types::sql_types::{Array, Record};
use diesel::sql_types::{Integer, VarChar};
use crate::diesel::RunQueryDsl;
use crate::schema::video_plays::dsl::video_plays;
use crate::schema::videos_tags::dsl::{videos_tags, tag_id};
use crate::schema::tags::dsl::tags;
use crate::models::{Tag, VideoWithUser, get_safe_user_fields};
use crate::schema::channels_tokens::dsl::channels_tokens;
use crate::schema::video_upvotes::dsl::{video_upvotes, video_id, upvote_type};
use crate::schema::users::dsl::users;
use crate::diesel::GroupByDsl;
use diesel::expression::dsl::not;

pub async fn get_recommended_videos(user: i32) -> Vec<VideoWithUser> {
    let db = establish_connection();

    // Get tags videos watched by the user
    //
    let tags_result: QueryResult<Vec<Tag>> = videos
        .select(
        sql::<Array<Record<(Integer, VarChar)>>>("array_remove(array_agg(distinct tags), null) as tags")
        )
        .inner_join(video_plays)
        .inner_join(videos_tags.on(crate::schema::videos_tags::video_id.eq(id)))
        .inner_join(tags.on(crate::schema::tags::id.eq(tag_id)))
        .filter(crate::schema::video_plays::user_id.eq(user))
        .first::<Vec<Tag>>(&db);

    // Get X most viewed videos that have a tag in that list
    let result: Vec<VideoWithUser> = videos
        .filter(not(
            exists(
                video_plays.filter(
                    crate::schema::video_plays::user_id.eq(user)
                        .and(crate::schema::video_plays::video_id.eq(crate::schema::videos::id)
                        )
                )
            )
        ))
        .filter(status.eq("READY"))
        .inner_join(users)
        .left_join(videos_tags.on(crate::schema::videos_tags::video_id.eq(crate::schema::videos::id)))
        .inner_join(tags.on(crate::schema::videos_tags::tag_id.eq(crate::schema::tags::id)))
        .left_join(channels_tokens.on(crate::schema::channels_tokens::channel_user_id.eq(crate::schema::users::id)))
        .select(
            (
                crate::schema::videos::id,
                crate::schema::videos::file_name,
                get_safe_user_fields(),
                crate::schema::videos::title,
                crate::schema::videos::description,
                crate::schema::videos::upload_date,
                exists(video_upvotes.
                    filter(video_id.eq(id)
                        .and(upvote_type.eq("UP"))
                        .and(crate::schema::video_upvotes::inactive.eq(false))
                        .and(crate::schema::video_upvotes::user_id.eq(user))
                    )
                ),
                exists(video_upvotes
                    .filter(video_id.eq(id)
                        .and(upvote_type.eq("DOWN"))
                        .and(crate::schema::video_upvotes::inactive.eq(false))
                        .and(crate::schema::video_upvotes::user_id.eq(user))
                    )
                ),
                diesel::dsl::sql::<diesel::sql_types::Integer>("count_video_upvotes(videos.id)"),
                diesel::dsl::sql::<diesel::sql_types::Integer>("count_video_downvotes(videos.id)"),
                diesel::dsl::sql::<diesel::sql_types::Integer>("count_video_plays(videos.id)"),
                diesel::dsl::sql::<diesel::sql_types::Array<Record<(Integer, VarChar)>>>("array_agg(\"tags\".*) as tags")
            )
        )
        .distinct()
        .group_by((crate::schema::videos::id, crate::schema::users::id, crate::schema::channels_tokens::id))
        .load(&db)
        .expect("Query failed.");


    return if let Ok(tags_result) = tags_result {
        // This is needed as Rust Diesel dose not support HAVING clauses
        let tags_ids: Vec<i32> = tags_result.iter().map(|tag| tag.id).collect();
        let result: Vec<VideoWithUser> = result.into_iter().filter(|video| {
            for tag in video.tags.iter() {
                if tags_ids.contains(&tag.id) {
                    return true;
                }
            }

            return false;
        }).collect();

        result
    } else {
        result
    }
}