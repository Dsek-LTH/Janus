use crate::discord;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::web::Redirect;
use actix_web::{cookie::Key, get, web, App, Error, HttpResponse, HttpServer, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct OAuthReturn {
    code: String,
    state: String,
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

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .build(),
            )
            .service(index)
            .service(linked_role)
            .service(discord_oauth_callback)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

