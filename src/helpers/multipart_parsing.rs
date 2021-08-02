use std::collections::HashMap;
use std::io::Write;

use actix_multipart::Multipart;
use actix_web::web;
use futures::{StreamExt, TryStreamExt};
use serde::de::DeserializeOwned;
use uuid::Uuid;

pub struct MultipartFile {
    pub file: std::fs::File,
    pub path: String,
    pub ext: String,
}

pub struct ParsedMultipart<D> {
    pub files: HashMap<String, MultipartFile>,
    pub data: Option<D>, // JSON of the multipart
}

/*
    This function takes a type argument D which implements Deserialize and a Multipart.
    It returns a ParsedMultipart<D>.
*/
pub async fn attempt_parse_multipart<D: DeserializeOwned>(mut multipart: Multipart<>) -> Result<ParsedMultipart<D>, &'static str> {
    let mut mime_type_extensions: HashMap<&str, &str> = HashMap::new();
    mime_type_extensions.insert("video/mpeg", "mpeg");
    mime_type_extensions.insert("video/mp4", "mp4");
    mime_type_extensions.insert("image/png", "png");
    mime_type_extensions.insert("image/jpeg", "jpeg");
    mime_type_extensions.insert("video/mkv", "mkv");

    let mut parsed_multipart: ParsedMultipart<D> = ParsedMultipart {
        files: HashMap::new(),
        data: None,
    };

    while let Ok(Some(mut field)) = multipart.try_next().await {
        if field.content_type().to_string() == "application/json" {
            let mut data: Vec<u8> = Vec::new();

            while let Some(chunk) = field.next().await {
                let bytes = chunk.unwrap();
                data.append(&mut bytes.to_vec());
            }

            let data = std::str::from_utf8(&data).unwrap();
            let data: Option<D> = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(_) => { return Err("Failed to deserialize JSON"); }
            };

            parsed_multipart.data = data;
        } else {
            let mime = field.content_type().to_string();

            let file_ext = mime_type_extensions.get(mime.as_str()).unwrap();

            let uid = Uuid::new_v4();
            let filepath = format!("/tmp/{}.{}", &uid, file_ext); // Assuming Unix system

            let filepath_copy = filepath.clone();
            let mut file = web::block(|| std::fs::File::create(filepath_copy))
                .await
                .unwrap();

            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                file = web::block(move || file.write_all(&data).map(|_| file)).await.unwrap();
            }

            let content_disposition = field.content_disposition().unwrap();
            let name = content_disposition.parameters.get(0).unwrap();
            let name = name.as_name().unwrap();

            let multipart_file = MultipartFile {
                file,
                path: filepath,
                ext: file_ext.to_string(),
            };

            parsed_multipart.files.insert(name.to_string(), multipart_file);
        }
    }

    return Ok(parsed_multipart);
}