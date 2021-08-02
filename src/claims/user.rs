use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserClaim {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub exp: i64,
    pub user_type: String,
}