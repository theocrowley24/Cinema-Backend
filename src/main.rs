// START Diesel imports

#[macro_use]
extern crate diesel;
extern crate dotenv;

use std::env;
use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use cronjob::CronJob;
use diesel::pg::PgConnection;
use diesel::prelude::*;

use crate::claims::user::UserClaim;
use crate::jobs::channel_payouts::convert_tokens;
use crate::jobs::assign_tokens::assign_tokens;

// END Diesel imports

mod claims;
mod middleware;
mod routes;
mod helpers;
mod schema;
mod models;
mod jobs;

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub struct AppState {
    pub user: Option<UserClaim>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // TOOD: fix this
    let mut convert_tokens_cron = CronJob::new("Convert tokens", convert_tokens);
    convert_tokens_cron.day_of_month("1"); // First of every month
    convert_tokens_cron.hours("12");
    convert_tokens_cron.minutes("00");
    convert_tokens_cron.seconds("0");
    convert_tokens_cron.offset(0);

    let mut assign_tokens_cron = CronJob::new("Assign tokens", assign_tokens);
    assign_tokens_cron.day_of_month("1");
    assign_tokens_cron.hours("12");
    assign_tokens_cron.minutes("30");
    assign_tokens_cron.seconds("0");
    assign_tokens_cron.offset(0);

    CronJob::start_job_threaded(convert_tokens_cron);
    CronJob::start_job_threaded(assign_tokens_cron);

    let state = web::Data::new(Mutex::new(AppState {
        user: None
    }));

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .service(
                web::scope("/auth")
                    .service(routes::auth::login)
                    .service(routes::auth::register)
                    .service(routes::auth::request_password_reset)
                    .service(routes::auth::reset_password)
                    .service(routes::auth::stripe_account_updated_hook)
                // .wrap(middleware::auth::CheckLogin {
                //     state: state.clone()
                // })
                // .service(routes::auth::is_channel_onboarded)
            )
            .service(
                web::scope("/video")
                    .wrap(middleware::auth::CheckLogin {
                        state: state.clone()
                    })
                    .service(routes::video::update_video)
                    .service(routes::video::get_available_tags)
                    .service(routes::video::get_popular_tags)
                    .service(routes::video::get_videos)
                    .service(routes::video::get_video)
                    .service(routes::video::record_play)
            )
            .service(
                web::scope("/upload")
                    .wrap(middleware::auth::CheckLogin {
                        state: state.clone()
                    })
                    .route("/", web::post().to(routes::upload::upload_video))
            )
            .service(
                web::scope("/tokens")
                    .wrap(middleware::auth::CheckLogin {
                        state: state.clone()
                    })
                    .service(routes::tokens::get_my_tokens)
                    .service(routes::tokens::transfer_token_to_channel)
                    .service(routes::tokens::get_active_tokens)
                    .service(routes::tokens::has_active_token)
                    .service(routes::tokens::get_my_balance)
                    .service(routes::tokens::get_my_transaction_history)
                    .service(routes::tokens::generate_withdrawal)
                    .service(routes::tokens::generate_account_link)
            )
            .service(
                web::scope("/comments")
                    .wrap(middleware::auth::CheckLogin {
                        state: state.clone()
                    })
                    .service(routes::comments::create_comment)
                    .service(routes::comments::edit_comment)
                    .service(routes::comments::delete_comment)
                    .service(routes::comments::get_comments)
            )
            .service(
                web::scope("/upvote")
                    .wrap(middleware::auth::CheckLogin {
                        state: state.clone()
                    })
                    .service(routes::upvotes::toggle_comment_upvote)
                    .service(routes::upvotes::toggle_video_upvote)
                    .service(routes::upvotes::get_video_upvote_count)
            )
            .service(
                web::scope("/users")
                    .wrap(middleware::auth::CheckLogin {
                        state: state.clone()
                    })
                    .service(routes::users::get_top_channels)
                    .service(routes::users::get_users)
                    .service(routes::users::get_user)
                    .service(routes::users::update_user)
            )
    })
        .bind("127.0.0.1:5000")?
        .run()
        .await
}