-- # STOCK

-- stock_id   | ticker | title
-------------------------------------------
-- 0123456789 | NVDA   | Nvidia Corporation
CREATE TABLE IF NOT EXISTS stock.index (
    stock_id    CHAR(10) PRIMARY KEY,
    ticker      VARCHAR(8),
    title       VARCHAR(255)
);

-- stock_id   | date_id  | dated      | metric   | val
--------------------------------------------------------------
-- 0123456789 | 20220101 | 2022-01-01 | Revenues | 249812378.0
CREATE TABLE IF NOT EXISTS stock.price (
    stock_id    CHAR(10),
    ts   	TIMESTAMP,
    opening     FLOAT,
    high        FLOAT,
    low         FLOAT,
    closing     FLOAT,
    adj_close   FLOAT,
    volume      INT
);

-- stock_id   | date_id  | dated      | metric   | val
--------------------------------------------------------------
-- 0123456789 | 20220101 | 2022-01-01 | Revenues | 249812378.0
CREATE TABLE IF NOT EXISTS stock.metrics (
    stock_id    CHAR(10),
    ts       	TIMESTAMP,
    metric      VARCHAR,
    val         FLOAT
);

-- # CRYPTO
