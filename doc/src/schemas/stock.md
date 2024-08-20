# CouchDB

## stock/index
```json
{
  "_id": "index",
  "_rev": "26-a5f2a5d668161591d2bcff4dee2bc64f",
  "data": [
    {
      "cik_str": "0000320193",
      "ticker": "AAPL",
      "title": "Apple Inc."
    },
    {
      "cik_str": "0000789019",
      "ticker": "MSFT",
      "title": "MICROSOFT CORP"
    },
    // ...
  ]
}
```
> **source** : "https://www.sec.gov/files/company_tickers.json"

## stock/{TICKER}
Using `stock/AAPL`:
```json
{
  "_id": "AAPL",
  "_rev": "27-f2357921b0e458d483c1f0a747efe881",
  "data": {
    "core": [
        {
            "MetricOne": 213123.0,
            "MetricTwo": 32423.0,
            // ...
            "MetricX": 0.5,
        },
        // ...
    ],
    "price": [
        {
            "adj_close": 143.85025024414062,
            "close": 146.08999633789062,
            "dated": "2021-08-09",
            "high": 146.6999969482422,
            "low": 145.52000427246094,
            "open": 146.1999969482422,
            "volume": 48908700
        },
        // ...
    ],
  }
}
```
> **source**
> - `price`: "https://query1.finance.yahoo.com/v8/finance/chart/AAPL?symbol=AAPL&interval=1d&range=3y&events=div|split|capitalGains"
> - `core`: "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip" <-- *this is a 1.1gb bulk .zip file*

# PostgreSQL

## stock.index
```sql
CREATE TABLE stock.index IF NOT EXISTS (
    pk SERIAL PRIMARY KEY,
    ticker VARCHAR,
    title VARCHAR
);
```

## stock.cores
```sql
CREATE TABLE stock.prices IF NOT EXISTS (
    pk SERIAL PRIMARY KEY,
    "MetricOne" NUMERIC,
    "MetricTwo" NUMERIC,
    -- ...
    "MetricX" NUMERIC,
);
```

## stock.prices
```sql
CREATE TABLE stock.prices IF NOT EXISTS (
    pk SERIAL PRIMARY KEY,
    high NUMERIC,
    open NUMERIC,
    low NUMERIC,
    close NUMERIC,
    adj_close NUMERIC,
    volume BIGINT
);
```