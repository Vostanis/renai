-- ---------------------------------------

-- # Tables
-- ----------
-- 1. stock.(index | prices | metrics)
-- 2. crypto.(index | prices)
-- 3. econ.(us)

-- ----------------------------------------

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
    stock_id    CHAR(10) NOT NULL,
    time   	TIMESTAMP WITH TIME ZONE NOT NULL,
    interval    CHAR(3) NOT NULL,
    opening     FLOAT,
    high        FLOAT,
    low         FLOAT,
    closing     FLOAT,
    adj_close   FLOAT,
    volume      BIGINT,
    PRIMARY KEY (stock_id, time, interval)
);

-- stock_id   | date_id  | dated      | metric   | val
-- -----------------------------------------------------------
-- 0123456789 | 20220101 | 2022-01-01 | Revenues | 249812378.0
CREATE TABLE IF NOT EXISTS stock.metrics (
    stock_id    CHAR(10) NOT NULL,
    dated       DATE NOT NULL,
    metric      VARCHAR NOT NULL,
    val         FLOAT NOT NULL,
    unit	VARCHAR NOT NULL,
    taxonomy    VARCHAR NOT NULL,
    PRIMARY KEY (stock_id, dated, metric, val, unit, taxonomy)
);

-- stock_id   | filename          | url                   | text 
-- ----------------------------------------------------------------
-- 0123456789 | nvda-20240101.htm | http://www.sec.gov... | ...
CREATE TABLE IF NOT EXISTS stock.filings {
    stock_id    CHAR(10) NOT NULL,
    filename    VARCHAR NOT NULL,
    filetype    VARCHAR,
    url         VARCHAR NOT NULL,
    text        LONGTEXT NOT NULL,
    PRIMARY KEY (stock_id, filename)
};

-- ## CRYPTO
-- crypto | pair
-- ----------------
-- 1      | BTCUSD
CREATE TABLE IF NOT EXISTS crypto.index (
    crypto_id   INT PRIMARY KEY,
    pair        VARCHAR(10)
);

-- crypto_id | dated      | opening | high | closing | low  | adj_close | volume | trades | source
-- ---------------------------------------------------------------------------------------------
-- 1         | 2022-01-01 | 1234    | 1234 | 1234    | 1234 | 1234      | 1234   | 321    | Binance
CREATE TABLE IF NOT EXISTS crypto.prices (
    crypto_id	INT NOT NULL, -- e.g., BTC
    time        TIMESTAMP WITH TIME ZONE NOT NULL,
    interval 	CHAR(2) NOT NULL,
    opening     FLOAT,
    high        FLOAT,
    low         FLOAT,
    closing     FLOAT,
    volume      FLOAT,
    trades	BIGINT,
    amount	FLOAT,
    source	CHAR(10), -- e.g., Binance, Coinbase, etc.
    PRIMARY KEY (crypto_id, time, interval, source)
);

-- ## ECON
-- dated      | metric       | val
-- ---------------------------------
-- 2022-01-01 | unemployment | 1234
CREATE TABLE IF NOT EXISTS econ.us (
    dated       DATE NOT NULL,
    metric	VARCHAR NOT NULL,
    val         FLOAT NOT NULL,
    PRIMARY KEY (dated, metric, val)
);
