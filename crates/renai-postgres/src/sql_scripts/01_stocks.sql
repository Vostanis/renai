CREATE SCHEMA IF NOT EXISTS stock;

CREATE TABLE IF NOT EXISTS stock.index (
    stock_id    CHAR(10) PRIMARY KEY,
    ticker      VARCHAR(8),
    title       VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS stock.price (
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

-- stock_id | date_id  | dated      | metric   | val
-------------------------------------------------------------
-- NVDA     | 20220101 | 2022-01-01 | Revenues | 249812378.0
CREATE TABLE IF NOT EXISTS stock.metrics (
    stock_id    CHAR(10),
    date_id     CHAR(8),
    dated       VARCHAR, -- needs to be DATE; use chrono::DateTime
    metric      VARCHAR,
    val         FLOAT
);

-- Essentially a hash table of metric names, including frequency of usage
CREATE TABLE IF NOT EXISTS stock.metric_ids (
    metric_id   INT,
    metric_name VARCHAR,    -- e.g., "Revenues", "DilutedEPS", etc.
    freq        INT         -- how often the metric name is used (across all stocks)
);