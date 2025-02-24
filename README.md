# Janus

Service to link Discord accounts with Keycloak with Discord role links

## Setup

1. Use Rust nightly
2. Initialize sqlite database with `sqlite3 database.db < tables.sql`
3. Populate the example.env file with info from your discord app and rename it to .env

It is recommended to use [ngrok](https://ngrok.com) to properly forward
traffic whilst testing.
