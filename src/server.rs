use actix_web::{web, get, App, Error, HttpResponse, Result, HttpServer, cookie::Key};
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::web::Redirect;
use serde::Deserialize;
use crate::discord;


#[derive(Deserialize)]
struct OAuthReturn {
    code: String,
    state: String
}


#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("hej"))
}


#[get("/linked-role")]
async fn linked_role(session: Session) -> Result<Redirect> {
    let discord::DiscordOAuthData { oauth_url, uuid_state } = discord::generate_oauth_url().await;
    session.insert("uuid_state", uuid_state)?;

    Ok(Redirect::to(oauth_url).temporary())
}

fn forbidden(body: String) -> Result<HttpResponse> {
    Ok(HttpResponse::Forbidden().body(body))
}

#[get("/discord-oauth-callback")]
async fn discord_oauth_callback(session: Session, oauth_return: web::Query<OAuthReturn>) -> Result<HttpResponse> {
    if let Some(uuid) = session.get::<String>("uuid_state")? {
        if uuid != oauth_return.state {
            return forbidden("Oauth state not same as cached state.\nWho are you...".to_string())
        }
    } else {
        return forbidden("Oauth state not found.\nWho are you...".to_string())
    }

    let oath_tokens = discord::get_oauth_tokens(oauth_return.code.clone()).await;

    dbg!(&oath_tokens);

    Ok(HttpResponse::Ok().body(oauth_return.code.clone()))
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    HttpServer::new(|| App::new()
        .wrap(
            SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .build()
        )
        .service(index)
        .service(linked_role)
        .service(discord_oauth_callback))
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
