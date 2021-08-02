use serde::Serialize;

use crate::schema::channels_tokens;
use crate::schema::comment_upvotes;
use crate::schema::comments;
use crate::schema::token_transactions;
use crate::schema::tokens;
use crate::schema::users;
use crate::schema::video_plays;
use crate::schema::video_upvotes;
use crate::schema::videos;
use crate::schema::videos_tags;
use crate::schema::users::columns::{user_type, id, username, avatar_filename, cover_filename, subscriptions_enabled, display_name, bio, channel_onboarded};
use diesel::types::{FromSql};
use diesel::backend::{Backend};
use diesel::pg::Pg;
use diesel::deserialize;
use diesel::sql_types::{Record, VarChar, Integer, BigInt};
use diesel::expression::SqlLiteral;

#[derive(Queryable, Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
    pub password_reset_token: Option<String>,
    pub user_type: String,
    pub stripe_customer: String,
    pub subscribed: bool,
    pub stripe_account: Option<String>,
    pub channel_onboarded: bool,
    pub avatar_filename: Option<String>,
    pub cover_filename: Option<String>,
    pub subscriptions_enabled: bool,
    pub display_name: Option<String>,
    pub bio: Option<String>
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub email: &'a str,
    pub user_type: &'a str,
}

#[derive(Queryable, Serialize)]
pub struct Video {
    pub id: i32,
    pub file_name: String,
    pub user_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub upload_date: std::time::SystemTime,
    pub status: String,
}

#[derive(Queryable, Serialize)]
pub struct SafeUser {
    pub id: i32,
    pub username: String,
    pub user_type: String,
    pub avatar_filename: Option<String>,
    pub cover_filename: Option<String>,
    pub subscriptions_enabled: bool,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub subscribers: i64,
    pub channel_onboarded: bool
}

// TODO: is there a nicer way to do this?
pub fn get_safe_user_fields() -> (id, username, user_type, avatar_filename, cover_filename, subscriptions_enabled, display_name, bio, SqlLiteral<BigInt, ()>, channel_onboarded) {
    (
        crate::schema::users::id,
        crate::schema::users::username,
        crate::schema::users::user_type,
        crate::schema::users::avatar_filename,
        crate::schema::users::cover_filename,
        crate::schema::users::subscriptions_enabled,
        crate::schema::users::display_name,
        crate::schema::users::bio,
        diesel::dsl::sql::<diesel::sql_types::BigInt>("count(\"channels_tokens\".*)"),
        crate::schema::users::channel_onboarded
    )
}

#[derive(Queryable, Serialize)]
pub struct VideoWithUser {
    pub id: i32,
    pub file_name: String,
    pub user: SafeUser,
    pub title: String,
    pub description: Option<String>,
    pub upload_date: std::time::SystemTime,
    pub upvoted: bool,
    pub downvoted: bool,
    pub upvotes: i32,
    pub downvotes: i32,
    pub plays: i32,
    pub tags: Vec<Tag>,
}

#[derive(Insertable)]
#[table_name = "videos"]
pub struct NewVideo<'a> {
    pub file_name: &'a str,
    pub user_id: i32,
    pub title: &'a str,
    pub description: Option<&'a str>,
}

#[derive(Queryable, Serialize)]
pub struct VideoTag {
    pub id: i32,
    pub video_id: i32,
    pub tag_id: i32,
}

#[derive(Insertable)]
#[table_name = "videos_tags"]
pub struct NewVideoTag<'a> {
    pub video_id: &'a i32,
    pub tag_id: &'a i32,
}

#[derive(Queryable, Serialize, SqlType)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Serialize)]
pub struct Token {
    pub id: i32,
    pub user_id: i32,
    pub used: bool,
    pub date_granted: std::time::SystemTime,
    pub date_used: Option<std::time::SystemTime>,
}

#[derive(Insertable)]
#[table_name = "tokens"]
pub struct NewToken {
    pub user_id: i32
}

#[derive(Queryable, Serialize)]
pub struct ChannelToken {
    pub id: i32,
    pub token_id: i32,
    pub channel_user_id: i32,
    pub expires: std::time::SystemTime,
    pub converted: bool,
}

#[derive(Queryable, Serialize)]
pub struct ChannelTokenAmount {
    pub channel_user_id: i32,
    pub token_count: i64,
}

#[derive(Queryable, Serialize)]
pub struct ChannelTokenWithUser {
    pub id: i32,
    pub token_id: i32,
    pub channel_user_id: i32,
    pub user: SafeUser,
    pub expires: std::time::SystemTime,
}

#[derive(Insertable)]
#[table_name = "channels_tokens"]
pub struct NewChannelToken {
    pub token_id: i32,
    pub channel_user_id: i32,
}

#[derive(Queryable)]
pub struct Comment {
    pub id: i32,
    pub user_id: i32,
    pub text: String,
    pub inactive: bool,
    pub date: std::time::SystemTime,
    pub video_id: i32,
}

#[derive(Queryable, Serialize)]
pub struct CommentWithUser {
    pub id: i32,
    pub user: SafeUser,
    pub text: String,
    pub inactive: bool,
    pub date: std::time::SystemTime,
    pub upvoted: bool,
    pub downvoted: bool,
    pub upvotes: i32,
    pub downvotes: i32,
}

#[derive(Insertable)]
#[table_name = "comments"]
pub struct NewComment {
    pub text: String,
    pub user_id: i32,
    pub video_id: i32,
}

#[derive(Queryable)]
pub struct CommentUpvote {
    pub id: i32,
    pub user_id: i32,
    pub comment_id: i32,
    pub inactive: bool,
    pub date: std::time::SystemTime,
    pub upvote_type: String,
}

#[derive(Insertable)]
#[table_name = "comment_upvotes"]
pub struct NewCommentUpvote {
    pub user_id: i32,
    pub comment_id: i32,
    pub upvote_type: String,
}

#[derive(Queryable)]
pub struct VideoUpvote {
    pub id: i32,
    pub user_id: i32,
    pub video_id: i32,
    pub inactive: bool,
    pub date: std::time::SystemTime,
    pub upvote_type: String,
}

#[derive(Insertable)]
#[table_name = "video_upvotes"]
pub struct NewVideoUpvote {
    pub user_id: i32,
    pub video_id: i32,
    pub upvote_type: String,
}

#[derive(Queryable)]
pub struct VideoPlay {
    pub id: i32,
    pub user_id: i32,
    pub video_id: i32,
    pub date: std::time::SystemTime,
}

#[derive(Insertable)]
#[table_name = "video_plays"]
pub struct NewVideoPlay {
    pub user_id: i32,
    pub video_id: i32,
}

#[derive(Queryable, Serialize)]
pub struct TokenTransaction {
    pub id: i32,
    pub channel_user_id: i32,
    pub transaction_type: String,
    pub amount: i32,
    pub date: std::time::SystemTime,
}

#[derive(Insertable)]
#[table_name = "token_transactions"]
pub struct NewTokenTransaction {
    pub channel_user_id: i32,
    pub transaction_type: String,
    pub amount: i32,
}

#[derive(Serialize, Queryable)]
pub struct PopularTag {
    pub id: i32,
    pub name: String,
    pub count: i64
}

#[derive(Serialize, Queryable)]
pub struct TopChannel {
    pub id: i32,
    pub username: String,
    pub user_type: String,
    pub avatar_filename: Option<String>,
    pub cover_filename: Option<String>,
    pub subscriptions_enabled: bool,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub plays: i64
}

impl FromSql<Record<(Integer, VarChar)>, diesel::pg::Pg> for Tag {
    fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> deserialize::Result<Self> {
        type DbTag = (i32, String);

        let val = DbTag::from_sql(bytes);

        match val {
            Ok(tag) => {
                Ok(Tag {
                    id: tag.0,
                    name: tag.1
                })
            }
            Err(_) => {
                Err("Could not convert to Tag object as bytes was None".into())
            }
        }
    }
}

