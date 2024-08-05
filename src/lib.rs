pub mod client_ext;
pub mod endp;
pub mod ui;
pub mod www;

use endp::{sec, yahoo_finance as yf};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Datasets {
    pub core: sec::Facts,
    pub price: Vec<yf::PriceCell>,
}
