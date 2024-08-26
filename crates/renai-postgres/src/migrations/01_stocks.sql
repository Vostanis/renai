CREATE SCHEMA IF NOT EXISTS stocks;

CREATE TABLE IF NOT EXISTS stocks.index (
    id          PRIMARY KEY,
    ticker      VARCHAR,
    title       VARCHAR,
);

CREATE TABLE IF NOT EXISTS stocks.price (
    id          PRIMARY KEY,
    opening     FLOAT,
    high        FLOAT,
    low         FLOAT,
    closing     FLOAT,
    adj_close   FLOAT,
);

CREATE TABLE IF NOT EXISTS stocks.core (
    id          PRIMARY KEY,
    packet      JSON,
);