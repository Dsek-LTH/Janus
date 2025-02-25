use serde::Deserialize;
use sqlx::sqlite::SqlitePool;

use crate::server::TokenData;

#[derive(Deserialize, Clone)]
pub struct SavedOAuthTokenData {
    pub access_token: String,
    pub expires_at: i64,
    pub refresh_token: String,
}

// FIXME: Both of these functions are heavily hacked together and require looking over
// I just did whatever I could to get shit working at all.

pub async fn get_token(user_id: &str) -> Option<SavedOAuthTokenData> {
    let db = SqlitePool::connect("sqlite:database.db").await.unwrap();
    let data: TokenData = sqlx::query_as::<_, TokenData>("SELECT * FROM tokens WHERE user_id = ?")
        .bind(&user_id)
        .fetch_one(&db)
        .await
        .unwrap();

    Some(SavedOAuthTokenData {
        access_token: data.access_token,
        expires_at: data.expires_at,
        refresh_token: data.refresh_token,
    })
}

pub async fn store_token(user_id: &str, oauth: SavedOAuthTokenData) {
    let data = TokenData {
        user_id: user_id.to_string(),
        access_token: oauth.access_token,
        refresh_token: oauth.refresh_token,
        expires_at: oauth.expires_at,
    };
    let db = SqlitePool::connect("sqlite:database.db").await.unwrap();
    sqlx::query(
        "INSERT OR REPLACE INTO tokens (user_id, access_token, refresh_token, expires_at) VALUES (?, ?, ?, ?)",
    )
    .bind(data.user_id)
    .bind(data.access_token)
    .bind(data.refresh_token)
    .bind(data.expires_at)
    .execute(&db)
    .await
    .unwrap();
}
