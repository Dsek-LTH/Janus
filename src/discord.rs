use dotenvy::dotenv;
use std::env;
use url_builder::URLBuilder;
use uuid::Uuid;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OAuthData {
    pub oauth_url: String,
    pub uuid_state: String
}

pub struct UserData {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String
}

#[derive(Serialize, Deserialize)]
struct OathTokenRequest {
    client_id: String,
    client_secret: String,
    grant_type: String,
    code: String,
    redirect_uri: String
}

pub async fn generate_oauth_url() -> OAuthData {
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

    OAuthData {
        oauth_url: url.build(),
        uuid_state: state
    }
}

pub async fn get_oauth_tokens(code: String) -> UserData {
    dotenv().ok();

    let client_id = env::var("CLIENT_ID").unwrap();
    let client_secret = env::var("CLIENT_SECRET").unwrap();
    let redirect_uri = env::var("DISCORD_REDIRECT_URI").unwrap();


    let metadata = OathTokenRequest {
        client_id: client_id,
        client_secret: client_secret,
        grant_type: "authorization_code".to_string(),
        code: code,
        redirect_uri: redirect_uri
    };

    let endpoint = "https://discord.com/api/v10/oauth2/token";
    let data = serde_urlencoded::to_string(&metadata).expect("serialize issue");
    println!("{}", &data);

    let client = reqwest::Client::new();

    UserData{
        access_token: "a".to_string(),
        token_type: "a".to_string(),
        expires_in: 100,
        refresh_token: "a".to_string(),
        scope: "a".to_string()
    }
}
