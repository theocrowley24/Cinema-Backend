use std::borrow::Borrow;
use std::sync::Mutex;

use actix_web::{get, HttpResponse, post, Responder, web};
use bcrypt::{hash, verify};
use diesel::{ExpressionMethods, QueryDsl, QueryResult};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use serde::Deserialize;
use stripe::{AccountType, CollectionMethod, RequestedCapability};
use validator::Validate;

use crate::{AppState, establish_connection};
use crate::claims::user;
use crate::diesel::RunQueryDsl;
use crate::helpers::stripe::create_account_link;
use crate::helpers::tokens::add_tokens;
use crate::models::{NewUser, User};
use crate::schema::users::columns::{channel_onboarded, email, id, password, password_reset_token, stripe_account, stripe_customer, username};
use crate::schema::users::dsl::users;

#[derive(Deserialize)]
pub struct LoginInfo {
    username: String,
    password: String,
}

#[post("/login")]
pub async fn login(data: web::Json<LoginInfo>) -> impl Responder {
    let db = establish_connection();
    let result: QueryResult<Vec<User>> = users.filter(username.eq(&data.username)).load::<User>(&db);
    let result = match result {
        Ok(v) => v,
        Err(_) => { return HttpResponse::BadRequest().json("User does not exist"); }
    };

    let user = match result.first() {
        Some(v) => v,
        None => { return HttpResponse::BadRequest().json("User does not exist"); }
    };

    let actual_password = &user.password;

    let valid = verify(&data.password, actual_password);
    let valid = match valid {
        Ok(v) => v,
        Err(_) => false,
    };

    if valid {
        let user_claim = user::UserClaim {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            exp: 10000000000,
            user_type: user.user_type.clone(),
        };

        let token = encode(&Header::default(), &user_claim, &EncodingKey::from_secret("secret".as_ref()));
        let token = match token {
            Ok(v) => v,
            Err(_) => {
                return HttpResponse::InternalServerError().json("Couldn't generate a JWT token. Sorry :(");
            }
        };

        return HttpResponse::Ok().json(token);
    }

    HttpResponse::BadRequest().json("Password incorrect!")
}

#[derive(Deserialize, Validate)]
pub struct RegisterInfo {
    #[validate(length(min = 1))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 1))]
    password: String,
    user_type: String,
    // TODO: validate this is either channel or subscriber
    payment_method_id: Option<String>,
}

#[post("/register")]
pub async fn register(data: web::Json<RegisterInfo>) -> impl Responder {
    match data.validate() {
        Ok(_) => (),
        Err(_) => {
            return HttpResponse::BadRequest().body("Invalid username, email or password");
        }
    }

    let hashed_password = hash(&data.password, 4);
    let hashed_password = match hashed_password {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Sorry, something went wrong on our end. Please try again.");
        }
    };

    let db = establish_connection();

    let new_user = NewUser {
        username: &data.username,
        password: &hashed_password,
        email: &data.email,
        user_type: &data.user_type,
    };

    let result: QueryResult<Vec<i32>> = diesel::insert_into(users)
        .values(&new_user)
        .returning(id)
        .get_results(&db);

    match result {
        Ok(_) => (),
        Err(_) => {
            return HttpResponse::BadRequest().body("Something went wrong. Please try again.");
        }
    };

    let secret_key = std::env::var("STRIPE_SECRET").expect("Missing STRIPE_SECRET_KEY in env");
    let client = stripe::Client::new(secret_key);

    let user_ids = result.unwrap();
    let user_id = user_ids.get(0).unwrap();

    if &data.user_type == "SUBSCRIBER" {
        // Create a new client

        let payment_method_id = data.payment_method_id.as_ref().unwrap();

        // Create the customer
        let mut params = stripe::CreateCustomer::new();
        params.email = Some(&data.email);
        params.payment_method = Some(payment_method_id.parse().unwrap()); // TODO: possibly not needed
        let customer = stripe::Customer::create(&client, params).await.unwrap();


        diesel::update(users.find(user_id))
            .set(stripe_customer.eq(customer.id.as_str()))
            .execute(&db)
            .expect("Couldn't add stripe_customer to user");

        // Subscription item
        let mut item = stripe::CreateSubscriptionItems::new();
        item.quantity = Some(1);
        item.price = Some(String::from("price_1IQztpIahEIGROhzWnYhQv1I")); // TODO: .env file

        // Params for the subscription
        let mut sub_params = stripe::CreateSubscription::new(customer.id);
        sub_params.collection_method = Some(CollectionMethod::ChargeAutomatically);
        sub_params.items = Some(vec![item]);

        let payment_method_id_copy = payment_method_id.clone();

        sub_params.default_payment_method = Some(payment_method_id_copy.as_str());

        stripe::Subscription::create(&client, sub_params)
            .await
            .expect("Failed to create subscription.");

        // TODO: assign tokens
        add_tokens(*user_id, 5);
    } else {
        // Channel signup

        // Create stripe account

        let mut params = stripe::CreateAccount::new();
        params.type_ = Option::from(AccountType::Express);
        params.country = Option::from("GB");
        params.email = Option::from(data.email.as_str());
        params.requested_capabilities = Some(vec![RequestedCapability::CardPayments, RequestedCapability::Transfers]);

        let account = stripe::Account::create(&client, params).await.unwrap();

        diesel::update(users.find(user_id))
            .set(stripe_account.eq(account.id.as_str()))
            .execute(&db)
            .expect("Couldn't add stripe_customer to user");

        // Create onboard link

        let result = create_account_link(&*account.id).await;

        return match result {
            Ok(url) => {
                HttpResponse::Ok().json(url)
            }
            Err(e) => {
                HttpResponse::BadRequest().json(e)
            }
        };
    }

    return HttpResponse::Ok().json("Registered");
}

#[derive(Deserialize, Validate)]
pub struct RequestPasswordResetInfo {
    #[validate(email)]
    email: String
}

#[post("/request-password-reset")]
pub async fn request_password_reset(data: web::Json<RequestPasswordResetInfo>) -> impl Responder {
    match data.validate() {
        Ok(_) => (),
        Err(_) => {
            return HttpResponse::BadRequest().body("Invalid email supplied.");
        }
    }

    let db = establish_connection();
    let result: QueryResult<Vec<User>> = users.filter(email.eq(&data.email)).load::<User>(&db);
    let result = match result {
        Ok(v) => v,
        Err(_) => { return HttpResponse::Ok().body("Done"); }
    };

    let user = match result.first() {
        Some(v) => v,
        None => { return HttpResponse::Ok().body("Done"); }
    };

    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    let hash = hash(&token, 4);

    let hash = match hash {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Sorry, something went wrong on our end. Please try again.");
        }
    };

    let update_result = diesel::update(users.find(user.id))
        .set(password_reset_token.eq(&hash))
        .execute(&db);

    return match update_result {
        Ok(_) => {
            HttpResponse::Ok().body(token) // TODO: send email with token to user instead of returning token
        }
        Err(_) => {
            HttpResponse::Ok().body("Success.") // Security
        }
    };
}

#[derive(Deserialize, Validate)]
pub struct ResetPasswordInfo {
    #[validate(email)]
    email: String,
    token: String,
    new_password: String,
}

#[post("/reset-password")]
pub async fn reset_password(data: web::Json<ResetPasswordInfo>) -> impl Responder {
    match data.validate() {
        Ok(_) => (),
        Err(_) => {
            return HttpResponse::BadRequest().body("Invalid email supplied.");
        }
    };

    let db = establish_connection();
    let result: QueryResult<Vec<User>> = users.filter(email.eq(&data.email)).load::<User>(&db);
    let result = match result {
        Ok(v) => v,
        Err(_) => { return HttpResponse::Ok().body("User doesn't exist."); }
    };

    let user = match result.first() {
        Some(v) => v,
        None => { return HttpResponse::Ok().body("User doesn't exist."); }
    };

    let actual_password_reset_token = match &user.password_reset_token {
        Some(v) => v,
        None => { return HttpResponse::BadRequest().body("User has not requested their password to be reset."); }
    };

    let valid = verify(&data.token, actual_password_reset_token);

    let valid = match valid {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Something went wrong on our end. Please try again.");
        }
    };

    return if valid {
        let new_password_hash = hash(&data.new_password, 4);
        let new_password_hash = match new_password_hash {
            Ok(v) => v,
            Err(_) => {
                return HttpResponse::InternalServerError().body("Something went wrong on our end. Please try again.");
            }
        };

        let update_result = diesel::update(users.find(user.id))
            .set((password_reset_token.eq(""), password.eq(new_password_hash)))
            .execute(&db);


        match update_result {
            Ok(_) => {
                HttpResponse::Ok().body("Password updated.")
            }
            Err(_) => {
                HttpResponse::NotFound().body("User does not exist.")
            }
        }
    } else {
        HttpResponse::Forbidden().body("Incorrect token supplied.")
    };
}

#[derive(Deserialize)]
pub struct Object {
    pub object: stripe::Account
}

#[derive(Deserialize)]
pub struct OnBoardCompleteWebhookBody {
    pub data: Object
}

#[post("/account-updated")]
pub async fn stripe_account_updated_hook(data: web::Json<OnBoardCompleteWebhookBody>) -> impl Responder {
    let email_ = data.data.object.email.as_ref().unwrap();
    let payout_enabled = data.data.object.payouts_enabled.unwrap();
    let db = establish_connection();

    if payout_enabled {
        let update_result = diesel::update(users.filter(email.eq(email_)))
            .set(channel_onboarded.eq(true))
            .execute(&db);

        return match update_result {
            Ok(_) => {
                HttpResponse::Ok().json("Success")
            }
            Err(_) => {
                HttpResponse::NotFound().json("User does not exist.")
            }
        };
    }

    HttpResponse::Ok().json("Payouts not enabled")
}

#[get("/onboarded")]
pub async fn is_channel_onboarded(state: web::Data<Mutex<AppState>>) -> impl Responder {
    let db = establish_connection();
    let state = state.lock().unwrap();
    let user = state.user.borrow().as_ref().unwrap();

    let result: User = users.find(user.id).first::<User>(&db).unwrap();

    HttpResponse::Ok().json(result.channel_onboarded)
}