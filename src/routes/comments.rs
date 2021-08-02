use std::borrow::Borrow;
use std::sync::Mutex;

use actix_web::{get, HttpResponse, post, Responder, web};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, JoinOnDsl};
use diesel::dsl::exists;
use crate::diesel::GroupByDsl;
use serde::Deserialize;
use validator::Validate;

use crate::{AppState, establish_connection};
use crate::models::{CommentWithUser, NewComment, get_safe_user_fields};
use crate::schema::comment_upvotes::dsl::{comment_id, comment_upvotes, upvote_type};
use crate::schema::comments::columns::{id, inactive, text, user_id, video_id};
use crate::schema::comments::dsl::comments;
use crate::schema::users::dsl::users;
use crate::schema::channels_tokens::dsl::channels_tokens;

#[derive(Deserialize, Validate)]
pub struct CreateCommentInfo {
    #[validate(length(min = 1, max = 256))]
    text: String,
    video: i32,
}

#[post("/")]
pub async fn create_comment(data: web::Json<CreateCommentInfo>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let db = establish_connection();

    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let new_comment = NewComment {
        text: data.text.to_owned(),
        user_id: user.id,
        video_id: data.video,
    };

    let result = diesel::insert_into(comments)
        .values(new_comment)
        .execute(&db);

    return match result {
        Ok(_) => HttpResponse::Ok().json("Comment added"),
        Err(_) => HttpResponse::BadRequest().body("Failed to add comment")
    };
}

#[derive(Deserialize)]
pub struct EditCommentInfo {
    text: String,
    comment: i32,
}

#[post("/edit")]
pub async fn edit_comment(data: web::Json<EditCommentInfo>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let db = establish_connection();

    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let result = diesel::update(
        comments.filter(
            id.eq(data.comment).and(user_id.eq(user.id))))
        .set(text.eq(data.text.to_owned()))
        .execute(&db);

    return match result {
        Ok(_) => HttpResponse::Ok().json("Comment updated"),
        Err(_) => HttpResponse::BadRequest().body("Failed to update comment")
    };
}

#[derive(Deserialize)]
pub struct DeleteCommentInfo {
    comment: i32
}

#[post("/delete")]
pub async fn delete_comment(data: web::Json<DeleteCommentInfo>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let db = establish_connection();

    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let result = diesel::update(
        comments.filter(
            id.eq(data.comment).and(user_id.eq(user.id))))
        .set(inactive.eq(true))
        .execute(&db);

    return match result {
        Ok(_) => HttpResponse::Ok().json("Comment deleted"),
        Err(_) => HttpResponse::BadRequest().body("Failed to delete comment")
    };
}

#[derive(Deserialize)]
pub struct GetCommentsParams {
    video_id: i32
}

#[get("/{video_id}")]
pub async fn get_comments(params: web::Path<GetCommentsParams>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let db = establish_connection();

    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    // sql_function!(fn count_comment_upvotes(c_id: Integer) -> Integer);
    // sql_function!(fn count_comment_downvotes(c_id: Integer) -> Integer);

    let result: Vec<CommentWithUser> = comments
        .inner_join(users)
        .left_join(channels_tokens.on(crate::schema::channels_tokens::channel_user_id.eq(crate::schema::users::id)))
        .select(
            (
                crate::schema::comments::id,
                get_safe_user_fields(),
                crate::schema::comments::text,
                crate::schema::comments::inactive,
                crate::schema::comments::date,
                exists(comment_upvotes
                    .filter(comment_id.eq(id)
                        .and(upvote_type.eq("UP"))
                        .and(crate::schema::comment_upvotes::inactive.eq(false))
                        .and(crate::schema::comment_upvotes::user_id.eq(user.id))
                    )
                ),
                exists(comment_upvotes
                    .filter(comment_id.eq(id)
                        .and(upvote_type.eq("DOWN"))
                        .and(crate::schema::comment_upvotes::inactive.eq(false))
                        .and(crate::schema::comment_upvotes::user_id.eq(user.id))
                    )
                ),
                diesel::dsl::sql::<diesel::sql_types::Integer>("count_comment_upvotes(comments.id)"),
                diesel::dsl::sql::<diesel::sql_types::Integer>("count_comment_downvotes(comments.id)"),
            )
        )
        .distinct()
        .filter(inactive.eq(false).and(video_id.eq(params.video_id)))
        .group_by((crate::schema::channels_tokens::id, crate::schema::comments::id, crate::schema::users::id))
        .load::<CommentWithUser>(&db)
        .expect("Query failed.");

    return HttpResponse::Ok().json(result);
}