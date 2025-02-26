use actix_web::Result;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};

use crate::server::from_server;

#[derive(Deserialize, Clone)]
pub struct OAuthCredentials {
    pub access_token: String,
    pub expires_at: i64,
    pub refresh_token: String,
}

pub async fn get_token(db: &Pool<Sqlite>, user_id: &str) -> Result<OAuthCredentials> {
    let res = sqlx::query_as!(
        OAuthCredentials,
        "SELECT access_token, refresh_token, expires_at 
        FROM discord_tokens 
        WHERE user_id = ?",
        user_id
    )
    .fetch_one(db)
    .await.map_err(from_server)?;

    Ok(res)
}

pub async fn store_token(db: &Pool<Sqlite>, user_id: &str, oauth: OAuthCredentials) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO discord_tokens (user_id, access_token, refresh_token, expires_at) 
        VALUES (?, ?, ?, ?)",
        user_id,
        oauth.access_token,
        oauth.refresh_token,
        oauth.expires_at
    )
    .execute(db)
    .await.map_err(from_server)?;

    Ok(())
}
