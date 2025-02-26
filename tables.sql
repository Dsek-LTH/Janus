DROP TABLE IF EXISTS discord_tokens;

CREATE TABLE discord_tokens (
    user_id         TEXT NOT NULL,
    access_token    TEXT NOT NULL,
    refresh_token   TEXT NOT NULL,
    expires_at      INTEGER NOT NULL,
    PRIMARY KEY (user_id)
);
