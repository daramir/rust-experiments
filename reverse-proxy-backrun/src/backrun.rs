// use the anyhow crate for easy idiomatic error handling
use anyhow::Result;
use ethers::{prelude::*, abi::{Token }, utils::hex};
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

fn vec_to_arr<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

pub async fn backrun_call(provider_url: String, 
    chain_id: u64, 
    signer: Wallet<k256::ecdsa::SigningKey>, 
    latest_event_id: &u32
) -> Result<TransactionReceipt> {
   
    let mut wallet = signer;
    let wally_addy = wallet.address();
    println!("Signer identifies as {}", wally_addy);

    // connect to the network
    // provider = ...
    let provider = Provider::<Http>::try_from(provider_url)?;
    wallet = wallet.with_chain_id(chain_id); // IMPORTANT

    // connect the wallet to the provider
    let client = SignerMiddleware::new(provider, wallet);

    let action_method_signature_hash: &'static str = "e1fa7638";
    // let sighash_type: &ParamType = &ParamType::FixedBytes(4);
    // let uint_type: &ParamType = &ParamType::Uint(256);

    let res_action_method_signature_hex = hex::decode(action_method_signature_hash);
    // let signed = short_signature(&self.name, &params).to_vec();
    let action_method_signature_hex: Vec<u8> = res_action_method_signature_hex?; //my signed
    // let action_selector: Selector = vec_to_arr::<u8, 4>(action_method_signature_hex); //Selector not useful here
    let param_1 = Token::Uint(U256::from(*latest_event_id));
    let param_2 = Token::Uint(U256::from(2949_u32));
    
	let encoded = abi::encode(&[param_1, param_2]);
    // adapted from https://docs.rs/ethers/latest/ethers/abi/struct.Function.html#method.encode_input
	let abi_compliant_calldata:Bytes = action_method_signature_hex.into_iter().chain(encoded.into_iter()).collect::<Vec<u8>>().into();
    
    // const callData = utils.hexConcat([
    //     action_method_signature_hash,
    //     param_1_hex,
    //     param_2_hex,
    //   ]);


    //   const maxPriorityFeePerGas = parseUnits("177.77", "gwei"); // 115.9
    //   const txReq = {
    //     to: gameAddy,
    //     data: callData,
    //     // maxPriorityFeePerGas,
    //     // maxFeePerGas: maxPriorityFeePerGas.add(feeDataGasPrice),
    //     gasPrice: maxPriorityFeePerGas.add(feeDataGasPrice),
    //     chainId: 43114,
    //     nonce: txNonce,
    //   };
    //   let gasEstimate = 180420;
    //   let txResponse;

    // craft the transaction
    let tx = TransactionRequest::new().to(wally_addy)
    .data(abi_compliant_calldata)
    .value(1);

    println!("{:?}", tx);

    // send it!
    println!("Sending tx");
    let pending_tx = match client.send_transaction(tx, None).await {
        Ok(tx) => Ok(tx),
        Err(tx_err) => {
            eprintln!("Error sending tx {}", tx_err);
            Err(tx_err)
        },
    };
    println!("Sent tx. Here's the pending tx:");
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