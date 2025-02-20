use std::env;

use dotenvy::dotenv;

pub fn var(name: &str) -> String {
    dotenv().ok();

    env::var(name).unwrap_or_else(|_| panic!("Environment variable {name} not defined"))
}