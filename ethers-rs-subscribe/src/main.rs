use ethers::prelude::*;
use std::{time::Duration};

#[macro_use]
extern crate lazy_static;

mod settings;

lazy_static! {
    static ref CONFIG: settings::Settings =
        settings::Settings::new().expect("config can't be loaded");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ws = Ws::connect(CONFIG.avalanche.node_rpc.wss.clone()).await?;
    let provider = Provider::new(ws).interval(Duration::from_millis(100));
    // let mut sub = provider.subscribe_blocks().await?;
    println!(
        "Running with game address: {} and rules: {:?}",
        CONFIG.external_contracts.addresses.game, CONFIG.rules
    );

    // let _t1 = "9729a6fbefefc8f6005933898b13dc45c3a2c8b7".parse::<Address>().unwrap();
    let game_addy: Address = CONFIG.external_contracts.addresses.game.parse().unwrap();
    let event_sig_hash: Vec<H256> = vec![CONFIG.eth_logs.event_signature_hashes.some_event.parse::<H256>().unwrap()];
    let mut filter = Filter::new().address(ValueOrArray::Value(game_addy));
    filter = filter.topic0(ValueOrArray::Array(event_sig_hash));

    let mut sub = provider.subscribe_logs(&filter).await?;

    while let Some(eth_log) = sub.next().await {
        // println!("block: {:?}", block.block_hash.unwrap_or_default());
        println!("tx: {:?}", eth_log.transaction_hash.unwrap_or_default());
    }

    Ok(())
}
