use anyhow::Result;
use std::future::Future;

pub trait Fetch {
    fn fetch_price(
        &self,
        ticker: &String,
        title: &String,
    ) -> impl Future<Output = Result<Vec<yf::PriceCell>>> + Send;

    fn mass_collection(&self) -> impl Future<Output = Result<()>> + Send;
}