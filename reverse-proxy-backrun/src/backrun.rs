// use the anyhow crate for easy idiomatic error handling
use anyhow::Result;
use ethers::{prelude::*};
use std::{path::Path};
// use the ethers_core rand for rng
// use ethers_core::rand::thread_rng;
// use the ethers_signers crate to manage LocalWallet and Signer
// use ethers_signers::{LocalWallet, Signer};

// async fn setup_random_wallet() -> Wallet<k256::ecdsa::SigningKey>{
//     return LocalWallet::new(&mut rand::thread_rng());
// }

pub fn setup_encrypted_json_wallet(keypath: &Path, password: String) -> Wallet<k256::ecdsa::SigningKey>{
    return Wallet::decrypt_keystore(keypath, password).unwrap();
}


// async fn backrun_filter() -> Result<()> {
//     // Generate a random wallet
//     let wallet = LocalWallet::new(&mut rand::thread_rng());

//     // Declare the message you want to sign.
//     let message = "Some data";

//     // sign message from your wallet and print out signature produced.
//     let signature = wallet.sign_message(message).await?;
//     println!("Produced signature {}", signature);

//     // verify the signature produced from your wallet.
//     signature.verify(message, wallet.address()).unwrap();
//     println!("Verified signature produced by {:?}!", wallet.address());

//     Ok(())
// }

pub async fn backrun_mine(provider_url: String, chain_id: u64, signer: Wallet<k256::ecdsa::SigningKey>) -> Result<TransactionReceipt> {
   
    let mut wallet = signer;
    let wally_addy = wallet.address();
    println!("Signer identifies as {}", wally_addy);

    // connect to the network
    // provider = ...
    let provider = Provider::<Http>::try_from(provider_url)?;
    wallet = wallet.with_chain_id(chain_id);

    // connect the wallet to the provider
    let client = SignerMiddleware::new(provider, wallet);

    // craft the transaction
    let tx = TransactionRequest::new().to(wally_addy).value(1);

    println!("{:?}", tx);

    // send it!
    let pending_tx = match client.send_transaction(tx, None).await {
        Ok(tx) => Ok(tx),
        Err(tx_err) => {
            eprintln!("Error sending tx {}", tx_err);
            Err(tx_err)
        },
    };

    println!("{:?}", pending_tx);

    // get the mined tx
    let receipt =
        pending_tx?
        .await?
        .ok_or_else(|| anyhow::format_err!("tx dropped from mempool"))?;
    let tx = client.get_transaction(receipt.transaction_hash).await?;

    println!("Sent tx: {}\n", serde_json::to_string(&tx)?);
    println!("Tx receipt: {}", serde_json::to_string(&receipt)?);

    Ok(receipt)
}