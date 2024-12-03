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

