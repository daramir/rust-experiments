// use the anyhow crate for easy idiomatic error handling
use anyhow::{Result, Context};
use ethers::abi::{ParamType, Token};
use ethers::prelude::*;
use ethers::prelude::{Address, Filter};
use ethers::types::{ValueOrArray, H256};
// use std::sync::{Arc, RwLock};
use std::sync::{Arc};
use std::{time::Duration};
use tokio::sync::*;

use crate::LatestEventCounter;
use crate::settings_mod::settings;


pub async fn start_event_listener(config: settings::Settings, latest_event_counter: Arc<RwLock<LatestEventCounter>> ) -> anyhow::Result<()> {
    let ws = Ws::connect(config.avalanche.mainnet_node_rpc.wss.clone()).await?;
    let provider = Provider::new(ws).interval(Duration::from_millis(100));
    // let mut sub = provider.subscribe_blocks().await?;
    println!(
        "Running with game address: {}",
        config.external_contracts.addresses.game
    );

    // let _t1 = "9729a6fbefefc8f6005933898b13dc45c3a2c8b7".parse::<Address>().unwrap();
    let game_addy: Address = config.external_contracts.addresses.game.parse().unwrap();
    let event_sig_hash: Vec<H256> = vec![config.eth_logs.event_signature_hashes.some_event.parse::<H256>().unwrap()];
    let mut filter = Filter::new().address(ValueOrArray::Value(game_addy));
    filter = filter.topic0(ValueOrArray::Array(event_sig_hash));

    let mut sub = provider.subscribe_logs(&filter).await?;
    let param_types: Vec<ParamType> = vec![ParamType::Uint(256), 
    ParamType::Uint(256), 
    ParamType::Uint(256), 
    ParamType::Uint(256), 
    ParamType::Uint(256)];

    while let Some(eth_log) = sub.next().await {
        // println!("block: {:?}", block.block_hash.unwrap_or_default());
        println!("tx: {:?}", eth_log.transaction_hash.unwrap_or_default());
        let log_data  = eth_log.data.as_ref();
        let tokenised_log_data: Vec<Token>  = ethers::abi::decode(&param_types, log_data).context("Tokenising log data")?;
        let action_id = tokenised_log_data[0].clone().into_uint().unwrap();
        let _actor_id  = tokenised_log_data[1].clone().into_uint().unwrap();
        latest_event_counter.write().await.latest_event_id = action_id.as_u32();
        println!("subscription updated action_id: {:?}", action_id);
    }

    // todo!()
    Ok(())
}