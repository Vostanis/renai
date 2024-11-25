-- Raw data tables, built via the following APIs:
	-- SEC EDGAR (fundamentals)
	-- Yahoo Finance (price)
CREATE SCHEMA IF NOT EXISTS stock;

-- Raw data tables, built via the following APIs:
	-- Binance
	-- Kraken
	-- KuCoin
	-- MEXC
	-- ByBit
CREATE SCHEMA IF NOT EXISTS crypto;

-- "Economic" data, such as GDP, unemployment, etc.
-- Built via the following APIs:
	-- FRED
CREATE SCHEMA IF NOT EXISTS econ;

-- "Queries"
-- Views, built off the backs off the raw-data tables (stock.price, etc.)
-- Used in production (analytics, applications, etc.)
CREATE SCHEMA IF NOT EXISTS q;
