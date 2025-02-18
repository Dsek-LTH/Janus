use std::env;

use dotenv::dotenv;
use serde_json::json;

#[tokio::main]
pub(crate) async fn register() {
    dotenv().ok();

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID environment variable missing");
    let token = env::var("BOT_TOKEN").expect("BOT_TOKEN environment variable missing");

    let metadata = json!([
        {
            "key": "dsekmember",
            "name": "Dsek Member",
            "description": "Member of the Dsek Guild",
            "type": 7
        }
    ]);

    let endpoint =
        format!("https://discord.com/api/v10/applications/{client_id}/role-connections/metadata");

    let client = reqwest::Client::new();
    let res = client
        .put(&endpoint)
        .json(&metadata)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bot {token}"))
        .send()
        .await;

    if let Ok(res) = res {
        if res.status().is_success() {
            println!("SUCCESS: {}", res.text().await.unwrap());
        } else {
            println!("ERROR: {}", res.text().await.unwrap());
        }
    }
}
