use crate::{env, storage};
use actix_session::Session;
use serde::Deserialize;
use std::collections::HashMap;
use url_builder::URLBuilder;
use uuid::Uuid;

// #[allow(unused)]
#[derive(Deserialize)]
pub struct OAuthTokenData {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
}

// #[allow(unused)]
#[derive(Deserialize)]
pub struct UserData {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: String,
    pub public_flags: u32,
}

// #[allow(unused)]
#[derive(Deserialize)]
pub struct AuthorizationData {
    pub expires: String,
    pub user: UserData,
}

pub async fn generate_oauth_url(session: &Session) -> String {
    let client_id = env::var("CLIENT_ID");
    let redirect_uri = env::var("DISCORD_REDIRECT_URI");

    let state = Uuid::new_v4().to_string();

    session
        .insert("uuid_state", &state)
        .expect("Could not insert state to session");

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

    url.build()
}

pub async fn fetch_oauth_tokens(code: &str) -> OAuthTokenData {
    let endpoint = "https://discord.com/api/v10/oauth2/token";
    let mut data = HashMap::new();

    data.insert("client_id", env::var("CLIENT_ID"));
    data.insert("client_secret", env::var("CLIENT_SECRET"));
    data.insert("redirect_uri", env::var("DISCORD_REDIRECT_URI"));
    data.insert("grant_type", "authorization_code".to_string());
    data.insert("code", code.to_string());

    reqwest::Client::new()
        .post(endpoint)
        .form(&data)
        .send()
        .await
        .expect("Something went wrong when sending OAuth token request to Discord")
        .json()
        .await
        .expect("Could not deserialize OAuth token data (check Discord docs for updates?)")
}

pub async fn fetch_user_auth_data(oauth: &OAuthTokenData) -> AuthorizationData {
    let endpoint = "https://discord.com/api/v10/oauth2/@me";

    reqwest::Client::new()
        .get(endpoint)
        .header("Authorization", format!("Bearer {}", oauth.access_token))
        .send()
        .await
        .expect("Something went wrong when fetching user data")
        .json()
        .await
        .expect("Could not deserialize Discord user data (check Discord docs for updates?)")
}

fn is_valid(ouath: &OAuthTokenData) -> bool {
    todo!()
}

async fn refresh_token(ouath: OAuthTokenData) -> OAuthTokenData {
    todo!()
}

pub async fn update_metadata(user_id: &str) {
    if let Some(mut oauth) = storage::get_token(user_id).await {
        if !is_valid(&oauth) {
            oauth = refresh_token(oauth).await;
        }

        storage::store_token(user_id, oauth);
    }
}
