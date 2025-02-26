use crate::{
    env,
    server::{from_server, AppState},
    storage::{self, OAuthCredentials},
};
use actix_session::Session;
use actix_web::{web, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url_builder::URLBuilder;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct UserDataResponse {
    pub id: String,
    pub username: String,
    // pub discriminator: String,
    // pub global_name: String,
    // pub public_flags: u32,
}

#[derive(Deserialize)]
pub struct AuthDataResponse {
    // pub expires: String,
    pub user: UserDataResponse,
}

#[derive(Serialize)]
struct MetadataUpdate {
    platform_name: String,
    platform_username: String,
    metadata: Metadata,
}
#[derive(Serialize)]
struct Metadata {
    dsek_member: bool,
}

pub fn generate_oauth_url(session: &Session) -> String {
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

pub async fn fetch_oauth_tokens(code: &str) -> Result<OAuthCredentials> {
    #[derive(Deserialize)]
    struct OAuthResponse {
        access_token: String,
        expires_in: i64,
        refresh_token: String,
    }

    impl From<OAuthResponse> for OAuthCredentials {
        fn from(oauth_resp: OAuthResponse) -> Self {
            OAuthCredentials {
                access_token: oauth_resp.access_token,
                refresh_token: oauth_resp.refresh_token,
                expires_at: Utc::now().timestamp() + oauth_resp.expires_in,
            }
        }
    }

    let endpoint = "https://discord.com/api/v10/oauth2/token";
    let mut data = HashMap::new();

    data.insert("client_id", env::var("CLIENT_ID"));
    data.insert("client_secret", env::var("CLIENT_SECRET"));
    data.insert("redirect_uri", env::var("DISCORD_REDIRECT_URI"));
    data.insert("grant_type", "authorization_code".to_string());
    data.insert("code", code.to_string());

    let res = reqwest::Client::new()
        .post(endpoint)
        .form(&data)
        .send()
        .await.map_err(from_server)?
        .json::<OAuthResponse>()
        .await.map_err(from_server)?
        .into();

    Ok(res)
}

pub async fn fetch_user_auth_data(access_token: &str) -> Result<AuthDataResponse> {
    let endpoint = "https://discord.com/api/v10/oauth2/@me";

    let res = reqwest::Client::new()
        .get(endpoint)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await.map_err(from_server)?
        .json::<AuthDataResponse>()
        .await.map_err(from_server)?;

    Ok(res)
}

async fn refresh_token(oauth: OAuthCredentials) -> Result<OAuthCredentials> {
    let endpoint = "https://discord.com/api/v10/oauth2/token";
    let mut data = HashMap::new();

    data.insert("client_id", env::var("CLIENT_ID"));
    data.insert("client_secret", env::var("CLIENT_SECRET"));
    data.insert("grant_type", "refresh_token".to_string());
    data.insert("refresh_token", oauth.refresh_token);

    let res = reqwest::Client::new()
        .post(endpoint)
        .form(&data)
        .send()
        .await.map_err(from_server)?
        .json()
        .await.map_err(from_server)?;

    Ok(res)
}

pub async fn update_metadata(data: &web::Data<AppState>, user_id: &str) -> Result<()> {
    let mut oauth = storage::get_token(&data.db, user_id).await?;
    if oauth.expires_at <= Utc::now().timestamp() {
        oauth = refresh_token(oauth).await?;
    }

    // TODO: Fetch proper metadata here
    let mdata = Metadata { dsek_member: true };
    let udata = MetadataUpdate {
        platform_name: "D-sektionen inom TLTH".to_string(),
        platform_username: user_id.to_string(), // should be stil-id once that's fetched, im guessing?
        metadata: mdata,
    };

    let endpoint = format!(
        "https://discord.com/api/v10/users/@me/applications/{}/role-connection",
        env::var("CLIENT_ID")
    );

    reqwest::Client::new()
        .put(endpoint)
        .json(&udata)
        .header("Authorization", format!("Bearer {}", oauth.access_token))
        .send()
        .await.map_err(from_server)?;

    storage::store_token(&data.db, user_id, oauth).await?;

    Ok(())
}
