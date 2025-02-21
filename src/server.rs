use crate::{discord, env};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::web::Redirect;
use actix_web::{cookie::Key, get, web, App, Error, HttpResponse, HttpServer, Result};
use serde::Deserialize;
use sqlx::{Pool, Sqlite, SqlitePool};

#[derive(Deserialize)]
struct OAuthReturn {
    code: String,
    state: String,
}

#[derive(Debug)]
struct TokenData {
    user_id: String,
    access_token: String,
    refresh_token: String
}

struct AppState {
    db: Pool<Sqlite>
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

    let oath_token = discord::fetch_oauth_tokens(&oauth_return.code).await;
    let auth_data = discord::fetch_user_auth_data(&oath_token).await;

    println!("{}", auth_data.user.username);

    Ok(HttpResponse::Ok().body(oauth_return.code.clone()))
}

#[get("/users")]
async fn list_users(data: web::Data<AppState>) -> Result<HttpResponse> {
    let res = sqlx::query_as!(TokenData, "SELECT * FROM tokens").fetch_all(&data.db).await.unwrap();

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
            .app_data(web::Data::new(AppState { db: storage.clone()}))
            .service(index)
            .service(linked_role)
            .service(list_users)
            .service(discord_oauth_callback)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
