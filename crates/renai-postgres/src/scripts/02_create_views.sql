-- ## STOCK
-- dated      | ticker | title              | opening | high | low | closing | adj_close | volume
-- ----------------------------------------------------------------------------------------------
-- 2021-01-01 | NVDA   | Nvidia Corporation | 1234    | 1234 | 1234| 1234    | 1234      | 1234
CREATE VIEW q.stock_prices AS (
SELECT
	dated,
	ticker,
	title,
	opening,
	high,
	low,
	closing,
	adj_close,
	volume
FROM	stock.prices	
	INNER JOIN stock.index
		USING(stock_id)
);

-- dated      | ticker | title              | metric   | val 
-- ----------------------------------------------------------
-- 2021-01-01 | NVDA   | Nvidia Corporation | Revenues | 1234
CREATE VIEW q.stock_metrics AS (
SELECT
	dated,
	ticker,
	title,
	metric,
	val
FROM	stock.metrics	
	INNER JOIN stock.index
		USING(stock_id)
);
