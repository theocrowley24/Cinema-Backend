use actix_web::{get, HttpResponse, Responder, web, post};
use diesel::{ExpressionMethods, QueryDsl, JoinOnDsl, TextExpressionMethods, BoolExpressionMethods};
use serde::Deserialize;

use crate::diesel::RunQueryDsl;
use crate::diesel::GroupByDsl;
use crate::{establish_connection, AppState};
use crate::models::{SafeUser, get_safe_user_fields, TopChannel};
use crate::schema::users::dsl::{id, avatar_filename, cover_filename, subscriptions_enabled, display_name, bio, password, user_type};
use actix_multipart::Multipart;
use std::sync::Mutex;
use std::borrow::Borrow;
use crate::helpers::multipart_parsing::attempt_parse_multipart;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use std::{fs, env};
use uuid::Uuid;
use crate::helpers::users::get_user_by_id;
use bcrypt::{verify, hash};
use crate::schema::videos::dsl::videos;
use crate::schema::users::dsl::users;
use crate::schema::video_plays::dsl::video_plays;
use crate::schema::users::columns::username;
use crate::schema::channels_tokens::dsl::channels_tokens;

#[derive(Deserialize)]
pub struct GetUserParams {
    pub user_id: i32
}

#[get("/{user_id}")]
pub async fn get_user(params: web::Path<GetUserParams>) -> impl Responder {
    let db = establish_connection();

    let result: Vec<SafeUser> = users
        .select(get_safe_user_fields())
        .filter(id.eq(params.user_id))
        .left_join(channels_tokens.on(crate::schema::channels_tokens::channel_user_id.eq(crate::schema::users::id)))
        .group_by(id)
        .load::<SafeUser>(&db).expect("Query failed.");

    if result.len() > 0 {
        return HttpResponse::Ok().json(result.get(0));
    }

    HttpResponse::NotFound().json("Not found")
}

#[derive(Deserialize)]
pub struct GetUsersBody {
    pub name: Option<String>
}

#[post("/")]
pub async fn get_users(body: web::Json<GetUsersBody>) -> impl Responder {
    let db = establish_connection();

    let mut query = users.into_boxed();
    query = query.filter(user_type.eq("CHANNEL")); // TOOD: change if you have time to optional body param

    if let Some(v) = &body.name {
        query = query.filter(username.like(format!("%{}%", v)).or(display_name.like(format!("%{}%", v))));
    }

    let result: Vec<SafeUser> = query
        .select(get_safe_user_fields())
        .left_join(channels_tokens.on(crate::schema::channels_tokens::channel_user_id.eq(crate::schema::users::id)))
        .group_by(id)
        .load::<SafeUser>(&db)
        .expect("Query failed");

    HttpResponse::Ok().json(result)
}

// TODO: validation
#[derive(Deserialize)]
pub struct UpdateChannelData {
    pub disable_subscriptions: Option<bool>,
    pub bio: Option<String>,
    pub display_name: Option<String>,
    pub current_password: Option<String>,
    pub new_password: Option<String>
}

#[post("/update-channel")]
pub async fn update_user(payload: Multipart, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let result = attempt_parse_multipart::<UpdateChannelData>(payload)
        .await;

    let result = match result {
        Ok(v) => v,
        Err(_) => { return HttpResponse::BadRequest().body("Couldn't parse multipart"); }
    };

    let bucket_name = &"cinema-storage";

    let region = Region::Custom {
        region: "us-east-1".to_string(),
        endpoint: "https://fra1.digitaloceanspaces.com".to_string()
    };

    let credentials = Credentials {
        access_key: Some(env::var("S3_KEY").expect("S3_KEY must be set")),
        secret_key: Some(env::var("S3_SECRET").expect("S3_SECRET must be set")),
        security_token: None,
        session_token: None
    };

    let mut bucket = Bucket::new(bucket_name, region, credentials).unwrap();
    bucket.add_header("x-amz-acl", "public-read");

    let db = establish_connection();

    let data = result.data.unwrap();

    match data.current_password {
        None => {}
        Some(cp) => {
            let db_user = get_user_by_id(&db, user.id).unwrap();

            let valid = verify(&cp, &*db_user.password);
            let valid = match valid {
                Ok(v) => v,
                Err(_) => false,
            };

            if !valid {
                return HttpResponse::BadRequest().json("Incorrect password supplied");
            }

            let new_password = data.new_password.unwrap();

            let hashed_password = hash(&new_password, 4).unwrap();

            diesel::update(users.find(user.id)).set(password.eq(&hashed_password))
                .execute(&db)
                .expect("Query failed");
        }
    }

    match data.bio {
        None => {}
        Some(b) => {
            diesel::update(users.find(user.id)).set(bio.eq(&b))
                .execute(&db)
                .expect("Query failed");
        }
    }

    match data.disable_subscriptions {
        None => {}
        Some(ds) => {
            diesel::update(users.find(user.id)).set(subscriptions_enabled.eq(&ds))
                .execute(&db)
                .expect("Query failed");
        }
    }

    match data.display_name {
        None => {}
        Some(dn) => {
            diesel::update(users.find(user.id)).set(display_name.eq(&dn))
                .execute(&db)
                .expect("Query failed");
        }
    }

    match result.files.get("avatar") {
        Some(avatar) => {
            let avatar_bytes = fs::read(avatar.path.to_owned()).unwrap();

            let uuid = Uuid::new_v4();

            let filename = format!("/images/avatars/{}.{}", uuid, avatar.ext);
            let db_filename = format!("{}.{}", uuid, avatar.ext);

            let (_, code) = bucket.put_object(filename,&avatar_bytes).await.unwrap();

            if code == 200 {
                diesel::update(users.find(user.id)).set(avatar_filename.eq(&db_filename))
                    .execute(&db)
                    .expect("Query failed");
            } else {
                return HttpResponse::BadRequest().json("Couldn't upload avatar.");
            }
        },
        None => ()
    };

    match result.files.get("cover") {
        Some(cover) => {
            let cover_bytes = fs::read(cover.path.to_owned()).unwrap();

            let uuid = Uuid::new_v4();

            let filename = format!("/images/covers/{}.{}", uuid, cover.ext);
            let db_filename = format!("{}.{}", uuid, cover.ext);

            let (_, code) = bucket.put_object(filename,&cover_bytes).await.unwrap();

            if code == 200 {
                diesel::update(users.find(user.id)).set(cover_filename.eq(&db_filename))
                    .execute(&db)
                    .expect("Query failed");
            } else {
                return HttpResponse::BadRequest().json("Couldn't upload avatar.");
            }
        },
        None => ()
    };

    HttpResponse::Ok().json("Channel updated")
}

#[get("/top-channels")]
pub async fn get_top_channels() -> impl Responder {
    let db = establish_connection();

    let result: Vec<TopChannel> = users
        .filter(user_type.eq("CHANNEL"))
        .select(
        (
            crate::schema::users::id,
            crate::schema::users::username,
            crate::schema::users::user_type,
            crate::schema::users::avatar_filename,
            crate::schema::users::cover_filename,
            crate::schema::users::subscriptions_enabled,
            crate::schema::users::display_name,
            crate::schema::users::bio,
            diesel::dsl::sql::<diesel::sql_types::BigInt>("count(\"video_plays\".*)")
        )
        )
        .left_join(videos)
        .left_join(video_plays.on(crate::schema::video_plays::video_id.eq(crate::schema::videos::id)))
        .left_join(channels_tokens.on(crate::schema::channels_tokens::channel_user_id.eq(crate::schema::users::id)))
        .group_by(crate::schema::users::id)
        .order_by(diesel::dsl::sql::<diesel::sql_types::BigInt>("count").desc())
        .limit(10)
        .load::<TopChannel>(&db)
        .expect("Query failed");

    HttpResponse::Ok().json(result)
}