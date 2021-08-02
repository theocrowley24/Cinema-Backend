use std::borrow::Borrow;
use std::sync::Mutex;

use actix_web::{get, HttpResponse, post, Responder, web};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, TextExpressionMethods, JoinOnDsl};
use diesel::dsl::exists;
use crate::diesel::GroupByDsl;
use serde::Deserialize;
use validator::Validate;

use crate::{AppState, establish_connection};
use crate::models::{NewVideoPlay, VideoWithUser, get_safe_user_fields, Tag, PopularTag};
use crate::schema::users::dsl::users;
use crate::schema::video_plays::dsl::video_plays;
use crate::schema::video_upvotes::dsl::{upvote_type, video_id, video_upvotes};
use crate::schema::videos::columns::{id, status, title, user_id, description};
use crate::schema::videos::dsl::videos;
use crate::schema::tags::dsl::tags;
use crate::schema::videos_tags::dsl::videos_tags;
use diesel::sql_types::{Integer, Record, VarChar};
use crate::schema::channels_tokens::dsl::channels_tokens;
use crate::helpers::recommender::get_recommended_videos;

#[derive(Deserialize)]
pub struct GetVideoParams {
    pub video_id: i32
}

#[get("/{video_id}")]
pub async fn get_video(params: web::Path<GetVideoParams>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let db = establish_connection();

    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let result: Vec<VideoWithUser> = videos
        .inner_join(users)
        // .inner_join(video_upvotes)
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
                        .and(crate::schema::video_upvotes::user_id.eq(user.id))
                    )
                ),
                exists(video_upvotes
                    .filter(video_id.eq(id)
                        .and(upvote_type.eq("DOWN"))
                        .and(crate::schema::video_upvotes::inactive.eq(false))
                        .and(crate::schema::video_upvotes::user_id.eq(user.id))
                    )
                ),
                diesel::dsl::sql::<diesel::sql_types::Integer>("count_video_upvotes(videos.id)"),
                diesel::dsl::sql::<diesel::sql_types::Integer>("count_video_downvotes(videos.id)"),
                diesel::dsl::sql::<diesel::sql_types::Integer>("count_video_plays(videos.id)"),
                diesel::dsl::sql::<diesel::sql_types::Array<Record<(Integer, VarChar)>>>("array_agg(\"tags\".*) as tags")
            )
        )
        .filter(id.eq(params.video_id))
        .group_by((crate::schema::videos::id, crate::schema::users::id, crate::schema::channels_tokens::id))
        .load::<VideoWithUser>(&db)
        .expect("Query failed");

    if result.len() > 0 {
        return HttpResponse::Ok().json(result.get(0));
    }

    HttpResponse::NotFound().json("Not found")
}

// TODO: valdiation here??
#[derive(Deserialize, Validate)]
pub struct GetVideosBody {
    pub title: Option<String>,
    pub tag: Option<i32>,
    pub user: Option<i32>,
    pub recommended: Option<bool>,
    pub subscriptions: Option<bool>,
    pub upvoted: Option<bool>,
    pub recently_watched: Option<bool>,
}

// TODO: recommended
// TODO: change filters to fiter_or
// TODO: pagination
#[post("/")]
pub async fn get_videos(data: web::Json<GetVideosBody>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let db = establish_connection();

    use crate::diesel::sql_types::Integer;

    sql_function!(fn video_has_tag(v_id: Integer, t_id: Integer) -> Bool);
    sql_function!(fn user_is_subscribed(u_id: Integer, c_id: Integer) -> Bool);
    sql_function!(fn video_recently_watched(u_id: Integer, v_id: Integer) -> Bool);

    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let mut query = videos.into_boxed();
    query = query.filter(status.eq("READY"));

    if let Some(v) = &data.title {
        // TODO: split into two?
        query = query.filter(title.like(format!("%{}%", v)).or(description.like(format!("%{}%", v))));
    }

    if let Some(v) = &data.tag {
        query = query.filter(video_has_tag(id, v));
    }

    if let Some(v) = &data.user {
        query = query.filter(user_id.eq(v));
    }

    if let Some(_) = &data.recommended {
        let result = get_recommended_videos(user.id).await;

        return HttpResponse::Ok().json(result);
    }

    if let Some(v) = &data.recently_watched {
        if v == &true {
            query = query.filter(video_recently_watched(user.id, id));
        }
    }

    if let Some(v) = &data.subscriptions {
        if v == &true {
            query = query.filter(user_is_subscribed(user.id, user_id));
        }
    }

    if let Some(v) = &data.upvoted {
        if v == &true {
            query = query.filter(exists(video_upvotes.
                filter(video_id.eq(id)
                    .and(upvote_type.eq("UP"))
                    .and(crate::schema::video_upvotes::inactive.eq(false))
                    .and(crate::schema::video_upvotes::user_id.eq(user.id))
                )
            ));
        }
    }

    let items: Vec<VideoWithUser> = query
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
                        .and(crate::schema::video_upvotes::user_id.eq(user.id))
                    )
                ),
                exists(video_upvotes
                    .filter(video_id.eq(id)
                        .and(upvote_type.eq("DOWN"))
                        .and(crate::schema::video_upvotes::inactive.eq(false))
                        .and(crate::schema::video_upvotes::user_id.eq(user.id))
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
        .load(&db).expect("Query failed.");

    return HttpResponse::Ok().json(items);
}

#[derive(Deserialize)]
pub struct RecordPlayBody {
    pub video: i32
}

#[post("/increment-play")]
pub async fn record_play(data: web::Json<RecordPlayBody>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let db = establish_connection();

    let new_video_play = NewVideoPlay {
        user_id: user.id,
        video_id: data.video,
    };

    let result = diesel::insert_into(video_plays)
        .values(new_video_play)
        .execute(&db);

    match result {
        Ok(_) => HttpResponse::Ok().json("Play recorded"),
        Err(_) => HttpResponse::BadRequest().json("Couldn't record play")
    }
}

#[get("/tags")]
pub async fn get_available_tags() -> impl Responder {
    let db = establish_connection();
    let result: Vec<Tag> = tags.load::<Tag>(&db).expect("Query failed");
    HttpResponse::Ok().json(result)
}



#[get("/popular-tags")]
pub async fn get_popular_tags() -> impl Responder {
    let db = establish_connection();

    let result: Vec<PopularTag> = tags.select(
        (
            crate::schema::tags::id,
            crate::schema::tags::name,
            diesel::dsl::sql::<diesel::sql_types::BigInt>("count(\"videos_tags\".*)")
        )
    )
        .left_join(videos_tags)
        .group_by(crate::schema::tags::id)
        .order_by(diesel::dsl::sql::<diesel::sql_types::BigInt>("count").desc())
        .limit(10)
        .load::<PopularTag>(&db)
        .expect("Query failed");

    HttpResponse::Ok().json(result)
}

#[derive(Deserialize)]
pub struct UpdateVideoBody {
    pub video: i32,
    pub title: Option<String>,
    pub description: Option<String>
}

#[post("/update")]
pub async fn update_video(data: web::Json<UpdateVideoBody>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let db = establish_connection();

    if let Some(t) = &data.title {
        diesel::update(
            videos.filter(id.eq(data.video).and(user_id.eq(user.id))))
            .set(title.eq(t))
            .execute(&db)
            .expect("Query failed");
    }

    if let Some(d) = &data.description {
        diesel::update(
            videos.filter(id.eq(data.video).and(user_id.eq(user.id))))
            .set(description.eq(d))
            .execute(&db)
            .expect("Query failed");
    }

    HttpResponse::Ok().json("Done")
}