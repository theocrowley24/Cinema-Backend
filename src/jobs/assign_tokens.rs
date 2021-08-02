use crate::establish_connection;
use crate::schema::users::dsl::{users, subscriptions_enabled};
use diesel::{QueryDsl, ExpressionMethods};
use crate::diesel::RunQueryDsl;
use crate::models::User;
use crate::helpers::tokens::add_tokens;

pub fn assign_tokens(name: &str) {
    println!("CRON JOB STARTED: {}", name);

    // Get list of all subscribers from database
    let db = establish_connection();

    let subscribers: Vec<User> = users
        .filter(subscriptions_enabled.eq(true))
        .load::<User>(&db)
        .expect("Query failed");

    for subscriber in subscribers {
        add_tokens(subscriber.id, 5);
    }

    println!("CRON JOB FINISHED: {}", name);
}