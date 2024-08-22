src/
	schema/
		crypto/
			price/
				binance.rs
				btse.rs
				coinbase.rs
				crypto_com.rs
				kraken.rs
				kucoin.rs
				mexc.rs
				mod.rs
			misc/
				btc.rs
				eth.rs
				kaspa.rs
				mod.rs
			mod.rs

		economic/
			aus/
				interest_rate.rs
				mod.rs
				unemployment.rs
			gr/
				interest_rate.rs
				mod.rs
				unemployment.rs
			uk/
				interest_rate.rs
				mod.rs
				unemployment.rs
			us/
				interest_rate.rs
				mod.rs
				unemployment.rs
			mod.rs

		people/
			gary_gensler/
				lawsuits.rs
				mod.rs
				twitter.rs
			warren_buffet/
				shareholder_letters.rs
				mod.rs
				twitter.rs
			mod.rs

		stocks/
			core/
				aus.rs
				gr.rs
				mod.rs
				uk.rs
				us.rs

			index/
				aus.rs
				gr.rs
				mod.rs
				uk.rs
				us.rs

			price/			<--- all comes from yahoo-finance
				fetch.rs
				mod.rs

			mod.rs
			fetch.rs

		mod.rs

	fetch.rs 				<--- impl Fetch for Client { fetch_all() }
	lib.rs					<--- pub mod prelude { ... }
	ui.rs
	util.rs					<--- read_json()
    
Cargo.lock
Cargo.toml