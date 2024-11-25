-- ## STOCK
-- dated      | ticker | title              | opening | high | low  | closing | adj_close | volume
-- ------------------------------------------------------------------------------------------------
-- 2021-01-01 | NVDA   | Nvidia Corporation | 1234    | 1234 | 1234 | 1234    | 1234      | 1234
DROP VIEW IF EXISTS q.stock_prices;
CREATE VIEW q.stock_prices AS (
SELECT
	time,
	ticker,
	title,
	opening,
	high,
	low,
	closing,
	adj_close,
	volume
FROM	
	stock.prices
	INNER JOIN stock.index
		USING(stock_id)
);

-- dated      | ticker | title              | metric   | val 
-- ----------------------------------------------------------
-- 2021-01-01 | NVDA   | Nvidia Corporation | Revenues | 1234
DROP VIEW IF EXISTS q.stock_metrics;
CREATE VIEW q.stock_metrics AS (
SELECT DISTINCT
	dated,
	ticker,
	title,
	metric,
	val,
	taxonomy
FROM	
	stock.metrics	
	INNER JOIN stock.index
		USING(stock_id)
);

-- dated      | ticker | title  | revenue | eps | earnings | free_cash_flow | debt | equity | debt_to_equity | outstanding_shares | stock_buybacks
-- ------------------------------------------------------------------------------------------------------------------------------------------------
-- 2021-01-01 | NVDA   | Nvidia | 1234    | 1.2 | 1234     | 1234           | 123  | 1234   | 1.1            | 123456789          | 123456789
-- DROP VIEW IF EXISTS q.stock_std_metrics;
-- CREATE VIEW q.stock_std_metrics AS (
-- SELECT
-- 	dated,
-- 	ticker,
-- 	title,
-- 	SUM(CASE WHEN metric IN ('Revenues', 'RevenueFromContractWithCustomerExcludingAssessedTax') THEN val END) AS revenue,
-- 	SUM(CASE WHEN metric IN ('NetIncomeLoss', 'ProfitLoss') THEN val END) as earnings,
-- 	-- earnings / revenue AS earnings_pct,
-- 	SUM(CASE WHEN metric = 'EarningsPerShareBasic' THEN val END) AS eps,
-- 	SUM(CASE WHEN metric = 'OperatingIncomeLoss' THEN val END) AS operating_income,
-- 	SUM(CASE WHEN metric = 'OperatingExpenses' THEN val END) AS operating_expenses,
-- 	SUM(CASE WHEN metric = 'ResearchAndDevelopmentExpense' THEN val END) AS research_and_dev,
-- 	SUM(CASE WHEN metric = 'SellingAndMarketingExpense' THEN val END) AS sales_and_marketing,
-- 	SUM(CASE WHEN metric = 'GeneralAndAdministrativeExpense' THEN val END) AS general_and_admin,
-- 	SUM(CASE WHEN metric = 'OtherDepreciationAndAmortization' THEN val END) AS amortization,
-- 	SUM(CASE WHEN metric = 'LongTermDebt' THEN val END) AS debt,
-- 	SUM(CASE WHEN metric = 'LongTermDebtCurrent' THEN val END) AS short_term_debt,
-- 	SUM(CASE WHEN metric = 'LongTermDebtNoncurrent' THEN val END) AS long_term_debt,
-- 	SUM(CASE WHEN metric = 'StockholdersEquity' THEN val END) AS equity,
-- 	-- debt / equity AS debt_to_equity,
-- 	SUM(CASE WHEN metric = 'EntityCommonStockSharesOutstanding' THEN val END) AS outstanding_shares,
-- 	SUM(CASE WHEN metric = 'PaymentsForRepurchaseOfCommonStock' THEN val END) AS stock_buyback_payments,
-- 	SUM(CASE WHEN metric = 'TreasuryStocksAcquired' THEN val END) AS stocks_acquired
-- 	-- revenue * outstanding_shares AS market_cap
-- FROM
-- 	q.stock_metrics
-- GROUP BY
-- 	dated,
-- 	ticker,
-- 	title
-- );
