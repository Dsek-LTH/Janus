use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use actix_web::web::Redirect;
use std::env;
use url_builder::URLBuilder;
use uuid::Uuid;

struct OAuthData {
    oauth_url: String,
    uuid_state: String
}

async fn generate_oauth_url() -> OAuthData {
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

#[get("/")]
async fn status() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/linked-role")]
async fn linked_role() -> impl Responder {
    let OAuthData { oauth_url, uuid_state } = generate_oauth_url().await;

    Redirect::to(oauth_url).temporary()
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(status).service(linked_role))
        .bind(("127.0.0.1", 3000))?
        .run()
        .await
}
