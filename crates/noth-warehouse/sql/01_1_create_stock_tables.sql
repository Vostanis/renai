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

-- stock_id   | filename          | filetype | url                   | text 
-- --------------------------------------------------------------------------
-- 0123456789 | nvda-20240101.htm | 10-Q     | http://www.sec.gov... | ...
CREATE TABLE IF NOT EXISTS stock.filings (
    stock_id    CHAR(10) NOT NULL,
    dated	DATE,
    filename    VARCHAR NOT NULL,
    filetype    VARCHAR,
    url         VARCHAR NOT NULL,
    content     TEXT NOT NULL,
    content_ts 	TSVECTOR NOT NULL,
    PRIMARY KEY (stock_id, filename)
);
