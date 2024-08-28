CREATE SCHEMA IF NOT EXISTS stocks;

CREATE TABLE IF NOT EXISTS stocks.index (
    pk_stocks   CHAR(10) PRIMARY KEY,
    ticker      VARCHAR(8),
    title       VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS stocks.price (
    pk_stocks   CHAR(10),
    dated       VARCHAR,
    opening     FLOAT,
    high        FLOAT,
    low         FLOAT,
    closing     FLOAT,
    adj_close   FLOAT,
    volume      INT
);

-- CREATE TABLE IF NOT EXISTS stocks.core (
--     pk_stocks   SERIAL PRIMARY KEY,
--     packet      JSON
-- );