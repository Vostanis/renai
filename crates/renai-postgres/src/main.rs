pub(crate) mod schema;

use schema::stock::price::url;

#[tokio::main]
async fn main() {
    println!("{}", url("NVDA"));
}

#[allow(dead_code)]
const RANDOM: &str = "https://query1.finance.yahoo.com/v8/finance/chart/NVDA?symbol=NVDA&interval=1d&range=3y&events=div|split|capitalGains";
