use std::borrow::Borrow;
use std::sync::Mutex;

use actix_multipart::Multipart;
use actix_web::{HttpResponse, Responder, web};
use diesel::{QueryResult, RunQueryDsl};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppState, establish_connection};
use crate::helpers::multipart_parsing::attempt_parse_multipart;
use crate::models::{NewVideo, NewVideoTag};
use crate::schema::videos::columns::id;
use crate::schema::videos::dsl::videos;
use crate::schema::videos_tags::dsl::videos_tags;

#[derive(Deserialize)]
pub struct UploadVideoData {
    video_title: String,
    video_description: Option<String>,
    video_tags: Vec<i32>,
}

// TODO: force user to supply at least one tag
pub async fn upload_video(payload: Multipart, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    if user.user_type != "CHANNEL" {
        return HttpResponse::Forbidden().body("Only channels can upload.");
    }

    let result = attempt_parse_multipart::<UploadVideoData>(payload)
        .await;

    let result = match result {
        Ok(v) => v,
        Err(_) => { return HttpResponse::BadRequest().body("Couldn't parse multipart"); }
    };

    let video = match result.files.get("video") {
        Some(v) => v,
        None => { return HttpResponse::BadRequest().body("No video found."); }
    };

    let thumbnail = match result.files.get("thumbnail") {
        Some(v) => v,
        None => { return HttpResponse::BadRequest().body("No thumbnail found."); }
    };

    let uuid = Uuid::new_v4();
    std::fs::create_dir(format!("./uploads/{}", uuid)).unwrap();

    std::fs::rename(&video.path, format!("./uploads/{}/source.{}", uuid, video.ext))
        .unwrap();

    std::fs::rename(&thumbnail.path, format!("./uploads/{}/thumbnail.{}", uuid, thumbnail.ext))
        .unwrap();

    let db = establish_connection();

    let data = result.data.unwrap();

    let new_video = NewVideo {
        file_name: &uuid.to_string(),
        user_id: user.id,
        title: &data.video_title,
        description: match &data.video_description {
            Some(v) => Some(v),
            None => None
        },
    };

    let pk: QueryResult<i32> = diesel::insert_into(videos)
        .values(&new_video)
        .returning(id)
        .get_result(&db);

    let pk = match pk {
        Ok(v) => v,
        Err(_) => { return HttpResponse::InternalServerError().body("Couldn't upload video"); }
    };

    for tag in data.video_tags {
        let new_video_tag = NewVideoTag {
            video_id: &pk,
            tag_id: &tag,
        };

        diesel::insert_into(videos_tags)
            .values(&new_video_tag)
            .execute(&db)
            .expect("Failed to add tag.");
    }

    HttpResponse::Ok().body("Uploaded")
}