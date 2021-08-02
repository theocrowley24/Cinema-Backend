use std::borrow::Borrow;
use std::sync::Mutex;

use actix_web::{HttpResponse, Responder, web};
use actix_web::{get, post};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use serde::Deserialize;
use serde::Serialize;
use crate::diesel::GroupByDsl;

use crate::{AppState, establish_connection};
use crate::helpers::stripe::{create_transfer, create_account_link};
use crate::helpers::tokens::{get_user_balance, get_user_transactions, transfer_token, user_has_active_token};
use crate::helpers::users::get_user_by_id;
use crate::models::{ChannelTokenWithUser, NewTokenTransaction, Token, get_safe_user_fields};
use crate::schema::channels_tokens::columns::expires;
use crate::schema::channels_tokens::dsl::{channel_user_id, channels_tokens, converted};
use crate::schema::token_transactions::dsl::token_transactions;
use crate::schema::tokens::dsl::{tokens, user_id};
use crate::schema::users::dsl::users;

#[get("/")]
pub async fn get_my_tokens(state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let db = establish_connection();
    let result: Vec<Token> = tokens
        .filter(user_id.eq(user.id))
        .load::<Token>(&db)
        .expect("Query failed");

    HttpResponse::Ok().json(result)
}

#[derive(Deserialize)]
pub struct TransferInfo {
    channel_user_id: i32
}

#[post("/transfer")]
pub async fn transfer_token_to_channel(data: web::Json<TransferInfo>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    // Check if the user is already subscribed
    let subscribed = user_has_active_token(user.id, data.channel_user_id);

    if subscribed {
        return HttpResponse::BadRequest().json("Already subscribed");
    }

    return match transfer_token(user.id, data.channel_user_id) {
        Ok(_) => HttpResponse::Ok().json("Done"),
        Err(e) => HttpResponse::BadRequest().body(e)
    };
}

#[get("/active")]
pub async fn get_active_tokens(state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let db = establish_connection();
    let result: Vec<ChannelTokenWithUser> = channels_tokens
        .inner_join(tokens)
        .inner_join(users.on(crate::schema::users::id.eq(crate::schema::channels_tokens::channel_user_id)))
        .select(
            (
                crate::schema::channels_tokens::id,
                crate::schema::channels_tokens::token_id,
                crate::schema::channels_tokens::channel_user_id,
                get_safe_user_fields(),
                crate::schema::channels_tokens::expires,
            )
        )
        .filter(user_id.eq(&user.id).and(expires.ge(std::time::SystemTime::now())))
        .group_by((crate::schema::channels_tokens::id, crate::schema::users::id))
        .load::<ChannelTokenWithUser>(&db)
        .expect("Query failed");

    HttpResponse::Ok().json(result)
}

#[derive(Deserialize)]
pub struct HasActiveTokenInfo {
    channel_id: i32
}

#[post("/has")]
pub async fn has_active_token(data: web::Json<HasActiveTokenInfo>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    // This handles are a channel viewing their own page
    if user.id == data.channel_id {
        return HttpResponse::Ok().json(true);
    }

    let active = user_has_active_token(user.id, data.channel_id);

    HttpResponse::Ok().json(active)
}

#[derive(Serialize)]
pub struct GetMyBalanceResponse {
    balance: i32,
    unconverted_tokens: i64,
}

#[get("/balance")]
pub async fn get_my_balance(state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let balance = get_user_balance(user.id);

    let db = establish_connection();

    // Get unconverted tokens
    let unconverted_tokens: Vec<i64> = channels_tokens
        .filter(channel_user_id.eq(user.id).and(converted.eq(false)))
        .select(diesel::dsl::sql::<diesel::sql_types::BigInt>("count(*)"))
        .load::<i64>(&db).unwrap();

    let unconverted_tokens = unconverted_tokens.get(0).unwrap();

    HttpResponse::Ok().json(GetMyBalanceResponse {
        balance,
        unconverted_tokens: *unconverted_tokens,
    })
}

#[get("/transaction-history")]
pub async fn get_my_transaction_history(state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let result = get_user_transactions(user.id);

    HttpResponse::Ok().json(result)
}

#[derive(Deserialize)]
pub struct GenerateWithdrawalBody {
    pub amount: i32
}

#[post("/generate-withdrawal")]
pub async fn generate_withdrawal(data: web::Json<GenerateWithdrawalBody>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let db = establish_connection();

    // Check balance is >= amount to withdraw
    let balance = get_user_balance(user.id);

    if balance < data.amount {
        return HttpResponse::BadRequest().json("You are withdrawing more than your current balance.");
    }

    // Check if Stripe Account is set up for withdrawals
    // If it isn't return error

    // Generate payout on Stripe
    let channel = get_user_by_id(&db, user.id).unwrap();

    match channel.stripe_account {
        None => {
            HttpResponse::BadRequest().json("You have not completed the on boarding process yet.")
        }
        Some(stripe_account_id) => {
            let transfer = create_transfer(stripe_account_id.as_str(), data.amount).await;

            match transfer {
                Ok(_) => {
                    // Insert WITHDRAWAL transaction into db
                    let new_token_transaction = NewTokenTransaction {
                        channel_user_id: user.id,
                        transaction_type: "WITHDRAWAL".to_string(),
                        amount: data.amount,
                    };

                    diesel::insert_into(token_transactions)
                        .values(new_token_transaction)
                        .execute(&db)
                        .expect("Query failed");

                    HttpResponse::Ok().json("Payout successful")
                }
                Err(_) => {
                    HttpResponse::BadRequest().json("Payout failed. Make sure you have completed the stripe onboarding process!")
                }
            }
        }
    }
}

#[get("/account-link")]
pub async fn generate_account_link(state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let db = establish_connection();

    let user = get_user_by_id(&db, user.id).unwrap();

    let stripe_account = match user.stripe_account {
        None => { return HttpResponse::BadRequest().json("Stripe account not created"); }
        Some(s_a) => s_a
    };



    let account_link = create_account_link(&*stripe_account).await;

    match account_link {
        Ok(url) => {
            HttpResponse::Ok().json(url)
        }
        Err(_) => {
            HttpResponse::BadRequest().json("Couldn't generate account link")
        }
    }
}