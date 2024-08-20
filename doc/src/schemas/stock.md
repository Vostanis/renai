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
    ...
  ]
}
```

## stock/{TICKER}
```json
{
  "_id": "AAPL",
  "_rev": "27-f2357921b0e458d483c1f0a747efe881",
  "data": {
    core: [
        {
            "MetricOne": 213123.0,
            "MetricTwo": 32423.0,
            ...
            "MetricX": 0.5,
        },
        ...
    ],
    price: [
        {
            "adj_close": 143.85025024414062,
            "close": 146.08999633789062,
            "dated": "2021-08-09",
            "high": 146.6999969482422,
            "low": 145.52000427246094,
            "open": 146.1999969482422,
            "volume": 48908700
        },
        ...
    ],
  }
}
```

# PostgreSQL

## stock.index
+-------------------------------------------------------+
| pk | ticker | title | [(industry)][1] | [(nation)][1] |
+-------------------------------------------------------+

## stock.cores
+--------------------------------------------+
| pk | MetricOne | MetricTwo | ... | MetricX |
+--------------------------------------------+

## stock.prices
+-----------------------------------------------------+
| pk | high | open | low | close | adj_close | volume |
+-----------------------------------------------------+

Note:
- [1] `()` denotes *columns to be added*, but are not yet available.