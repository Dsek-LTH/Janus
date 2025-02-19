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
async fn index(session: Session) -> Result<HttpResponse, Error> {
    // access session data
    let uuid = match session.get::<String>("uuid_state") {
        Ok(Some(uuid_state)) => {
            uuid_state
        }
        Ok(None) => {
           "None".to_string()
        }
        Err(_) => {
            "Error: {:?}".to_string()
        }
    };
    Ok(HttpResponse::Ok().body(format!(
        "state is {:?}!",
        uuid
    )))
}


#[get("/linked-role")]
async fn linked_role(session: Session) -> Result<Redirect> {
    let discord::OAuthData { oauth_url, uuid_state } = discord::generate_oauth_url().await;
    session.insert("uuid_state", uuid_state)?;

    Ok(Redirect::to(oauth_url).temporary())
}

#[get("/discord-oauth-callback")]
async fn discord_oauth_callback(session: Session, oauth_return: web::Query<OAuthReturn>) -> Result<HttpResponse, Error> {
    let uuid_state = match session.get::<String>("uuid_state") {
        Ok(Some(uuid)) => {
            uuid
        }
        Ok(None) => {
           "None".to_string()
        }
        Err(_) => {
            "Error: {:?}".to_string()
        }
    };

    if uuid_state != oauth_return.state {
        return Ok(HttpResponse::Forbidden().body("Oauth state not same as cashe safe.\nWho are you..."));
    }

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
