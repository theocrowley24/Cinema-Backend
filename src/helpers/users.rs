use diesel::{ExpressionMethods, PgConnection, QueryDsl};

use crate::diesel::RunQueryDsl;
use crate::models::User;
use crate::schema::users::columns::id;
use crate::schema::users::dsl::users;

pub fn get_user_by_id(db: &PgConnection, target_user_id: i32) -> Option<User> {
    let result: Vec<User> = users
        .filter(id.eq(&target_user_id))
        .load::<User>(db)
        .expect("Couldn't retrieve user");

    let result = result.into_iter().nth(0);

    return result;
}