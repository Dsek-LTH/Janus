use crate::{discord, env, storage};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::web::Redirect;
use actix_web::{cookie::Key, get, web, App, Error, HttpResponse, HttpServer, Result};
use chrono::offset::Utc;
use serde::Deserialize;
use sqlx::{Encode, FromRow, Pool, Sqlite, SqlitePool};

#[derive(Deserialize)]
struct OAuthReturn {
    code: String,
    state: String,
}

#[derive(Debug, FromRow, Encode)]
pub struct TokenData {
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

struct AppState {
    db: Pool<Sqlite>,
}

fn forbidden(body: &str) -> Result<HttpResponse> {
    Ok(HttpResponse::Forbidden().body(body.to_string()))
}

#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("hej"))
}

#[get("/linked-role")]
async fn linked_role(session: Session) -> Result<Redirect> {
    let url = discord::generate_oauth_url(&session).await;
    Ok(Redirect::to(url).temporary())
}

#[get("/discord-oauth-callback")]
async fn discord_oauth_callback(
    session: Session,
    oauth_return: web::Query<OAuthReturn>,
) -> Result<HttpResponse> {
    if let Some(state) = session.get::<String>("uuid_state")? {
        if state != oauth_return.state {
            return forbidden("Oauth state not same as cached state.\nWho are you...?");
        }
    } else {
        return forbidden("Oauth state not found.\nWho are you...?");
    }

    let oauth_token = discord::fetch_oauth_tokens(&oauth_return.code).await;
    let auth_data: discord::AuthorizationData = discord::fetch_user_auth_data(&oauth_token).await;

    let save_oauth_token = storage::SavedOAuthTokenData {
        expires_at: Utc::now().timestamp() + oauth_token.expires_in,
        access_token: oauth_token.access_token,
        refresh_token: oauth_token.refresh_token,
    };
    storage::store_token(&auth_data.user.id, save_oauth_token).await;
    discord::update_metadata(&auth_data.user.id).await;

    Ok(HttpResponse::Ok().body(format!(
        "Process completed for user {}",
        &auth_data.user.username
    )))
}

#[get("/users")]
async fn list_users(data: web::Data<AppState>) -> Result<HttpResponse> {
    let res = sqlx::query_as!(TokenData, "SELECT * FROM tokens")
        .fetch_all(&data.db)
        .await
        .unwrap();

    Ok(HttpResponse::Ok().body(format!("{res:?}")))
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let db_url = env::var("DATABASE_URL");
    let storage = SqlitePool::connect(&db_url)
        .await
        .expect("Could not connect to database");

    HttpServer::new(move || {
        let cookie_store =
            SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64])).build();

        App::new()
            .wrap(cookie_store)
            .app_data(web::Data::new(AppState {
                db: storage.clone(),
            }))
            .service(index)
            .service(linked_role)
            .service(list_users)
            .service(discord_oauth_callback)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
