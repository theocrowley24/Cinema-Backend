use std::time::SystemTime;

use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, QueryResult, RunQueryDsl};

use crate::establish_connection;
use crate::helpers::users::get_user_by_id;
use crate::models::{ChannelTokenWithUser, NewChannelToken, NewToken, Token, TokenTransaction, get_safe_user_fields};
use crate::schema::channels_tokens::dsl::channels_tokens;
use crate::schema::token_transactions::dsl::token_transactions;
use crate::schema::tokens::columns::date_used;
use crate::schema::tokens::dsl::{tokens, user_id};
use crate::schema::users::dsl::users;
use crate::diesel::GroupByDsl;

pub fn user_has_active_token(source_user_id: i32, target_channel_user_id: i32) -> bool {
    let db = establish_connection();
    let result: QueryResult<ChannelTokenWithUser> = channels_tokens
        .inner_join(tokens)
        .inner_join(users.on(crate::schema::users::id.eq(crate::schema::tokens::user_id)))
        .select(
            (
                crate::schema::channels_tokens::id,
                crate::schema::channels_tokens::token_id,
                crate::schema::channels_tokens::channel_user_id,
                get_safe_user_fields(),
                crate::schema::channels_tokens::expires,
            )
        )
        .filter(crate::schema::tokens::user_id.eq(source_user_id)
            .and(crate::schema::channels_tokens::channel_user_id.eq(target_channel_user_id))
            .and(crate::schema::channels_tokens::expires.ge(std::time::SystemTime::now()))
        )
        .group_by((crate::schema::channels_tokens::id, crate::schema::users::id))
        .first::<ChannelTokenWithUser>(&db);

    return match result {
        Ok(_) => {
            true
        },
        Err(_) => {
            false
        }
    };
}

pub fn add_tokens(target_user_id: i32, amount: u32) {
    let db = establish_connection();

    for _ in 0..amount {
        let new_token = NewToken {
            user_id: target_user_id
        };

        diesel::insert_into(tokens)
            .values(&new_token)
            .execute(&db)
            .expect("Failed to add token");
    }
}

// Returns Result of date_used
pub fn transfer_token(source_user_id: i32, channel_id: i32) -> Result<std::time::SystemTime, String> {
    let db = establish_connection();
    let result: Vec<Token> = tokens
        .filter(user_id.eq(&source_user_id).and(date_used.is_null()))
        .load::<Token>(&db)
        .expect("Query failed");

    if result.len() == 0 {
        return Err(String::from("You do not have any tokens left"));
    }

    let channel = get_user_by_id(&db, channel_id);
    let channel = match channel {
        Some(v) => v,
        None => { return Err(String::from("Channel does not exist")); }
    };

    if channel.user_type != "CHANNEL" {
        return Err(String::from("Target is not a channel"));
    }

    // CHeck if the user already has an active token
    let active = user_has_active_token(source_user_id, channel_id);

    if active {
        return Err(String::from("You already have a token"));
    }

    let token = result.get(0).unwrap();

    let new_channel_token = NewChannelToken {
        token_id: token.id,
        channel_user_id: channel_id,
    };

    diesel::insert_into(channels_tokens)
        .values(&new_channel_token)
        .execute(&db)
        .expect("Couldn't transfer token");

    let new_date_used = SystemTime::now();

    diesel::update(tokens.find(token.id))
        .set(date_used.eq(&new_date_used))
        .execute(&db)
        .expect("Couldn't mark token as used");

    return Ok(new_date_used);
}

pub fn get_user_balance(target_user_id: i32) -> i32 {
    let result = get_user_transactions(target_user_id);

    let mut balance = 0;

    for transaction in result {
        if transaction.transaction_type == "DEPOSIT" {
            balance += transaction.amount;
        } else if transaction.transaction_type == "WITHDRAWAL" {
            balance -= transaction.amount;
        }
    }

    return balance;
}

pub fn get_user_transactions(target_user_id: i32) -> Vec<TokenTransaction> {
    let db = establish_connection();

    let result: Vec<TokenTransaction> = token_transactions.filter(crate::schema::token_transactions::channel_user_id.eq(target_user_id))
        .load::<TokenTransaction>(&db)
        .expect("Query failed");

    return result;
}