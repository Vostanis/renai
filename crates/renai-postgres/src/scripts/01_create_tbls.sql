------------------------------------------

-- # CONTENTS
-- ----------
--
-- 1. stock.(index | prices | metrics)
-- 2. crypto.(prices)
-- 3. econ.(us)

-------------------------------------------

-- ## STOCK
-- stock_id   | ticker | title
-- ----------------------------------------
-- 0123456789 | NVDA   | Nvidia Corporation
CREATE TABLE IF NOT EXISTS stock.index (
    stock_id    CHAR(10) PRIMARY KEY,
    ticker      VARCHAR(8),
    title       VARCHAR(255),
    industry	VARCHAR(255),
    nation	VARCHAR(6)
);

-- stock_id   | dated 	   | opening | high | closing | low  | adj_close | volume
-- ------------------------------------------------------------------------------
-- 0123456789 | 2022-01-01 | 1234    | 1234 | 1234    | 1234 | 1234      | 1234
CREATE TABLE IF NOT EXISTS stock.prices (
    stock_id    CHAR(10),
    dated   	DATE,
    opening     FLOAT,
    high        FLOAT,
    low         FLOAT,
    closing     FLOAT,
    adj_close   FLOAT,
    volume      INT
);

-- stock_id   | date_id  | dated      | metric   | val
-- -----------------------------------------------------------
-- 0123456789 | 20220101 | 2022-01-01 | Revenues | 249812378.0
CREATE TABLE IF NOT EXISTS stock.metrics (
    stock_id    CHAR(10),
    dated       DATE,
    metric      VARCHAR,
    val         FLOAT,
    unit	VARCHAR,
    taxonomy    VARCHAR
);

-- ## CRYPTO
-- pair   | dated      | opening | high | closing | low  | adj_close | volume
-- --------------------------------------------------------------------------
-- BTCUSD | 2022-01-01 | 1234    | 1234 | 1234    | 1234 | 1234      | 1234
CREATE TABLE IF NOT EXISTS crypto.prices (
    ticker	VARCHAR, -- e.g., BTC
    dated       DATE,
    opening     FLOAT,
    high        FLOAT,
    low         FLOAT,
    closing     FLOAT,
    volume      FLOAT,
    trades	BIGINT,
    source	VARCHAR -- e.g., Binance, Coinbase, etc.
);

-- ## ECON
-- dated      | metric       | val
-- ---------------------------------
-- 2022-01-01 | unemployment | 1234
CREATE TABLE IF NOT EXISTS econ.us (
    dated       DATE,
    metric	VARCHAR,
    val         FLOAT
);
