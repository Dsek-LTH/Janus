use std::fmt::{Debug, Display};

use crate::{discord, env, storage};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key,
    error::InternalError,
    get,
    web::{self, Redirect},
    App, Error, HttpResponse, HttpServer, ResponseError, Result,
};
use serde::Deserialize;
use sqlx::{Pool, Sqlite, SqlitePool};

/// Converts any error into a 500 Internal Server Error response. 
pub fn from_server<T: Debug + Display>(err: T) -> impl ResponseError {
    InternalError::new(err, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Deserialize)]
struct OAuthReturn {
    code: String,
    state: String,
}

pub struct AppState {
    pub db: Pool<Sqlite>,
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
    let url = discord::generate_oauth_url(&session);
    Ok(Redirect::to(url).temporary())
}

#[get("/discord-oauth-callback")]
async fn discord_oauth_callback(
    session: Session,
    oauth_return: web::Query<OAuthReturn>,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    // I kind of want this to be its own function, makes it feel a bit cleaner
    if let Some(state) = session.get::<String>("uuid_state")? {
        if state != oauth_return.state {
            return forbidden("OAuth state not same as cached state.\nWho are you...?");
        }
    } else {
        return forbidden("OAuth state not found.\nWho are you...?");
    }

    let oauth_token = discord::fetch_oauth_tokens(&oauth_return.code).await?;
    let auth_data = discord::fetch_user_auth_data(&oauth_token.access_token).await?;

    storage::store_token(&data.db, &auth_data.user.id, oauth_token).await?;
    discord::update_metadata(&data, &auth_data.user.id).await?;

    Ok(HttpResponse::Ok().body(format!(
        "Process completed for user {}",
        &auth_data.user.username
    )))
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let db_url = env::var("DATABASE_URL");
    let storage = SqlitePool::connect(&db_url)
        .await
        .map_err(|_| std::io::ErrorKind::ConnectionRefused)?;

    HttpServer::new(move || {
        let cookie_store = {
            let mut key = [0; 64];
            // should it just be random? leaving it commented out for now
            // rand::rng().fill(&mut key);
            SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&key)).build()
        };

        let app_state = web::Data::new(AppState {
            db: storage.clone(),
        });

        App::new()
            .wrap(cookie_store)
            .app_data(app_state)
            .service(index)
            .service(linked_role)
            .service(discord_oauth_callback)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
