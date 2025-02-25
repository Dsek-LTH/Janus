DROP TABLE IF EXISTS tokens;

CREATE TABLE tokens (
    user_id         TEXT NOT NULL,
    access_token    TEXT NOT NULL,
    refresh_token   TEXT NOT NULL,
    expires_at      INTEGER NOT NULL,
    PRIMARY KEY (user_id)
);

INSERT INTO tokens
VALUES  ('a', 'some-token-1', 'really-long-value-1', 2),
        ('b', 'some-token-2', 'really-long-value-2', 4),
        ('c', 'some-token-3', 'really-long-value-3', 3);
