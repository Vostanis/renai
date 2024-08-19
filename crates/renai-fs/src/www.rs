const INTERVAL: &str = "1d";
const RANGE: &str = "3y";

pub async fn price_url(ticker: &str) -> String {
    format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{ticker}?symbol={ticker}&interval={}&range={}&events=div|split|capitalGains",
        INTERVAL,
        RANGE
    )
}
