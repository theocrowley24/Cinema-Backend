use std::borrow::Borrow;
use std::sync::Mutex;

use actix_web::{get, HttpResponse, post, Responder, web};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel::dsl::count_star;
use serde::Deserialize;
use serde::Serialize;

use crate::{AppState, establish_connection};
use crate::models::{CommentUpvote, NewCommentUpvote, NewVideoUpvote, VideoUpvote};
use crate::schema::comment_upvotes::dsl::comment_upvotes;
use crate::schema::video_upvotes::dsl::video_upvotes;

#[derive(Deserialize)]
pub struct ToggleCommentUpvoteInfo {
    pub comment: i32,
    pub upvote_type: String,
}

#[post("/toggle-comment")]
pub async fn toggle_comment_upvote(data: web::Json<ToggleCommentUpvoteInfo>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let db = establish_connection();

    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    // Has this user already upvoted / downvoted this comment?
    let result: Vec<CommentUpvote> = comment_upvotes
        .filter(crate::schema::comment_upvotes::comment_id.eq(data.comment)
            .and(crate::schema::comment_upvotes::user_id.eq(user.id)))
        .load::<CommentUpvote>(&db)
        .expect("Query failed.");

    let existing_upvote = result.get(0);

    match existing_upvote {
        Some(existing_upvote) => {
            // The user has already upvoted / downvoted this comment so we need to update
            // the record

            let mut inactive = false;

            if existing_upvote.upvote_type == data.upvote_type {
                inactive = !existing_upvote.inactive;
            }

            diesel::update(comment_upvotes.find(existing_upvote.id))
                .set(
                    (
                        crate::schema::comment_upvotes::upvote_type.eq(&data.upvote_type),
                        crate::schema::comment_upvotes::inactive.eq(inactive)
                    )
                )
                .execute(&db)
                .expect("Query failed.");
        }
        None => {
            let new_comment_upvote = NewCommentUpvote {
                user_id: user.id,
                comment_id: data.comment,
                upvote_type: String::from(&data.upvote_type),
            };

            let result = diesel::insert_into(comment_upvotes)
                .values(new_comment_upvote)
                .execute(&db);

            return match result {
                Ok(_) => {
                    HttpResponse::Ok().json("Upvoted")
                }
                Err(_) => {
                    HttpResponse::BadRequest().body("Comment does not exist")
                }
            };
        }
    }

    HttpResponse::Ok().json("Toggled upvote")
}

#[derive(Deserialize)]
pub struct ToggleVideoUpvoteInfo {
    pub video: i32,
    pub upvote_type: String,
}

#[post("/toggle-video")]
pub async fn toggle_video_upvote(data: web::Json<ToggleVideoUpvoteInfo>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let db = establish_connection();

    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    // Has this user already upvoted / downvoted this comment?
    let result: Vec<VideoUpvote> = video_upvotes
        .filter(crate::schema::video_upvotes::video_id.eq(data.video)
            .and(crate::schema::video_upvotes::user_id.eq(user.id)))
        .load::<VideoUpvote>(&db)
        .expect("Query failed.");

    let existing_upvote = result.get(0);

    match existing_upvote {
        Some(existing_upvote) => {
            // The user has already upvoted / downvoted this comment so we need to update
            // the record

            let mut inactive = false;

            if existing_upvote.upvote_type == data.upvote_type {
                inactive = !existing_upvote.inactive;
            }

            diesel::update(video_upvotes.find(existing_upvote.id))
                .set(
                    (
                        crate::schema::video_upvotes::upvote_type.eq(&data.upvote_type),
                        crate::schema::video_upvotes::inactive.eq(inactive)
                    )
                )
                .execute(&db)
                .expect("Query failed.");
        }
        None => {
            let new_video_upvote = NewVideoUpvote {
                user_id: user.id,
                video_id: data.video,
                upvote_type: String::from(&data.upvote_type),
            };

            let result = diesel::insert_into(video_upvotes)
                .values(new_video_upvote)
                .execute(&db);

            return match result {
                Ok(_) => {
                    HttpResponse::Ok().json("Upvoted")
                }
                Err(_) => {
                    HttpResponse::BadRequest().body("Video does not exist")
                }
            };
        }
    }

    HttpResponse::Ok().json("Toggled upvote")
}

#[derive(Deserialize)]
pub struct GetUpvoteCountParams {
    video_id: i32
}

#[derive(Serialize)]
pub struct GetUpvoteCountResponse {
    upvotes: i64,
    downvotes: i64,
}

#[get("/{video_id}")]
pub async fn get_video_upvote_count(params: web::Path<GetUpvoteCountParams>) -> impl Responder {
    let db = establish_connection();

    // Upvote count
    let upvotes: i64 = video_upvotes
        .filter(crate::schema::video_upvotes::video_id.eq(params.video_id)
            .and(crate::schema::video_upvotes::upvote_type.eq("UP")))
        .select(count_star())
        .first(&db)
        .expect("Query failed.");

    // Downvote count
    let downvotes: i64 = video_upvotes
        .filter(crate::schema::video_upvotes::video_id.eq(params.video_id)
            .and(crate::schema::video_upvotes::upvote_type.eq("DOWN")))
        .select(count_star())
        .first(&db)
        .expect("Query failed.");

    HttpResponse::Ok().json(GetUpvoteCountResponse {
        upvotes,
        downvotes,
    })
}
