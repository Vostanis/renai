CREATE SCHEMA IF NOT EXISTS crypto;

CREATE TABLE IF NOT EXISTS crypto.index (
    coin_id     CHAR(10) PRIMARY KEY,
    coin_pair   VARCHAR(8),
);

CREATE TABLE IF NOT EXISTS crypto.price (
    stock_id    CHAR(10),
    date_id     CHAR(8),
    dated       VARCHAR, -- needs to be DATE; use chrono::DateTime
    opening     FLOAT,
    high        FLOAT,
    low         FLOAT,
    closing     FLOAT,
    adj_close   FLOAT,
    volume      INT
);