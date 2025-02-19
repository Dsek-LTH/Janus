use dotenvy::dotenv;
use serde::Deserialize;
use std::{collections::HashMap, env};
use url_builder::URLBuilder;
use uuid::Uuid;

pub struct DiscordOAuthData {
    pub oauth_url: String,
    pub uuid_state: String,
}

#[derive(Deserialize, Debug)]
pub struct TokenData {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
}

pub async fn generate_oauth_url() -> DiscordOAuthData {
    dotenv().ok();

    let client_id = env::var("CLIENT_ID").unwrap();
    let redirect_uri = env::var("DISCORD_REDIRECT_URI").unwrap();

    let state = Uuid::new_v4().to_string();

    let mut url = URLBuilder::new();
    url.set_protocol("https")
        .set_host("discord.com")
        .add_route("/api/oauth2/authorize")
        .add_param("client_id", &client_id)
        .add_param("redirect_uri", &redirect_uri)
        .add_param("response_type", "code")
        .add_param("state", &state)
        .add_param("scope", "role_connections.write identify")
        .add_param("prompt", "consent");

    DiscordOAuthData {
        oauth_url: url.build(),
        uuid_state: state,
    }
}

pub async fn get_oauth_tokens(code: String) -> TokenData {
    dotenv().ok();

    let endpoint = "https://discord.com/api/v10/oauth2/token";
    let mut data = HashMap::new();

    data.insert("client_id", env::var("CLIENT_ID").unwrap());
    data.insert("client_secret", env::var("CLIENT_SECRET").unwrap());
    data.insert("redirect_uri", env::var("DISCORD_REDIRECT_URI").unwrap());
    data.insert("grant_type", "authorization_code".to_string());
    data.insert("code", code);

    reqwest::Client::new()
        .post(endpoint)
        .form(&data)
        .send()
        .await
        .expect("Something went wrong :(")
        .json::<TokenData>()
        .await
        .expect("Could not deserialize json (check discord docs for updates?)")
}
