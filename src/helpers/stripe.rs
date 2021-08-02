use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateAccountLinkResponse {
    url: String,
}

// TODO: handle errors better
// TODO: redirect URLS
pub async fn create_account_link(account_id: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let mut params = HashMap::new();
    params.insert("account", account_id);
    params.insert("refresh_url", "http://localhost:3000/login");
    params.insert("return_url", "http://localhost:3000");
    params.insert("type", "account_onboarding");

    let secret_key = std::env::var("STRIPE_SECRET").expect("Missing STRIPE_SECRET_KEY in env");

    let res = client.post("https://api.stripe.com/v1/account_links")
        .basic_auth(secret_key, Some(""))
        .form(&params)
        .send();

    return match res {
        Ok(res) => {
            let account_link = res.json::<CreateAccountLinkResponse>().unwrap();
            Ok(account_link.url)
        }
        Err(_) => {
            Err("Couldn't create account link".parse().unwrap())
        }
    };
}

#[derive(Deserialize)]
pub struct CreateTransferResponse {
}

pub async fn create_transfer(account_id: &str, amount: i32) -> Result<CreateTransferResponse, String> {
    let client = reqwest::blocking::Client::new();
    let mut params = HashMap::new();
    params.insert("destination", account_id);

    let amount = amount.to_string();
    let amount = amount.as_str();
    params.insert("amount", amount);
    params.insert("currency", "gbp");

    let secret_key = std::env::var("STRIPE_SECRET").expect("Missing STRIPE_SECRET_KEY in env");

    let res = client.post("https://api.stripe.com/v1/transfers")
        .basic_auth(secret_key, Some(""))
        .form(&params)
        .send();

    return match res {
        Ok(res) => {
            let res = res.json::<CreateTransferResponse>();

            match res {
                Ok(v) => {Ok(v)}
                Err(_) => {Err("Couldn't create transfer".parse().unwrap())}
            }
        }
        Err(_) => {
            Err("Couldn't create transfer".parse().unwrap())
        }
    };
}