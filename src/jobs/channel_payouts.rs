use diesel::{ExpressionMethods, QueryDsl};

use crate::diesel::GroupByDsl;
use crate::diesel::RunQueryDsl;
use crate::establish_connection;
use crate::models::{ChannelTokenAmount, NewTokenTransaction};
use crate::schema::channels_tokens::columns::{channel_user_id, converted};
use crate::schema::channels_tokens::dsl::channels_tokens;
use crate::schema::token_transactions::dsl::token_transactions;

// For each channel count the unconverted tokens they have
// Multiply the amount by the token value
// Insert into token_transactions the result as a DEPOSIT
pub fn convert_tokens(name: &str) {
    println!("CRON JOB STARTED: {}", name);

    let db = establish_connection();

    println!("Retrieving unconverted tokens");
    // Retrieve unconverted tokens for each channel
    let result: Vec<ChannelTokenAmount> = channels_tokens.filter(converted.eq(false))
        .select(
            (
                channel_user_id,
                diesel::dsl::sql::<diesel::sql_types::BigInt>("count(*)"),
            )
        )
        .group_by(channel_user_id)
        .load::<ChannelTokenAmount>(&db)
        .expect("Query failed");

    // Map the result into a insertable db models
    let transactions: Vec<NewTokenTransaction> = result.iter().map(|ct| {
        NewTokenTransaction {
            channel_user_id: ct.channel_user_id,
            transaction_type: "DEPOSIT".to_string(),
            amount: (ct.token_count * 180) as i32,
        }
    }).collect();

    println!("Depositing money to users");
    // Insert into db
    diesel::insert_into(token_transactions)
        .values(transactions)
        .execute(&db)
        .expect("Query failed");

    diesel::update(channels_tokens.filter(converted.eq(false)))
        .set(converted.eq(true))
        .execute(&db)
        .expect("Query failed");

    println!("CRON JOB FINISHED: {}", name);
}