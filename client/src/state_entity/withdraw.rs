//! Withdraw
//!
//! Withdraw funds from the state entity

// withdraw() messages:
// 0. request withdraw and provide withdraw tx data
// 1. co-sign withdraw tx
// 2. verify withdraws transaction received is corrcet

use super::super::Result;
extern crate shared_lib;
use shared_lib::{structs::{StateChainDataAPI, WithdrawMsg1, PrepareSignTxMsg, Protocol, WithdrawMsg2},
    state_chain::StateChainSig,
    util::tx_withdraw_build};

use crate::wallet::wallet::Wallet;
use crate::state_entity::util::cosign_tx_input;
use super::api::{get_statechain, get_statechain_fee_info};
use crate::utilities::requests;
use crate::error::CError;

use bitcoin::{PublicKey, consensus};
use curv::elliptic::curves::traits::ECPoint;

use std::str::FromStr;

/// Withdraw coins from state entity. Returns signed withdraw transaction, state_chain_id and withdrawn amount.
pub fn withdraw(wallet: &mut Wallet, shared_key_id: &String)
    -> Result<(String, String, u64)>
{
    // first get required shared key data
    let state_chain_id;
    let pk;
    {
        let shared_key = wallet.get_shared_key(shared_key_id)?;
        pk = shared_key.share.public.q.get_element();
        state_chain_id = shared_key.state_chain_id.clone()
            .ok_or(CError::Generic(String::from("No state chain for this shared key id")))?;
    }

     // Generate receiving address of withdrawn funds
    let rec_address = wallet.keys.get_new_address()?;

    // Sign state chain
    let state_chain_data: StateChainDataAPI = get_statechain(&wallet.client_shim, &state_chain_id)?;
    if state_chain_data.amount == 0 {
        return Err(CError::Generic(String::from("Withdraw: StateChain is already withdrawn.")));
    }
    let state_chain = state_chain_data.chain;
    // get proof key for signing
    let proof_key_derivation = wallet.se_proof_keys.get_key_derivation(&PublicKey::from_str(&state_chain.last().unwrap().data).unwrap());
    let state_chain_sig = StateChainSig::new(
        &proof_key_derivation.unwrap().private_key.key,
        &String::from("WITHDRAW"),
        &rec_address.to_string()
    )?;

    // Alert SE of desire of withdraw and receive authorisation if state chain signature verifies
    requests::postb(&wallet.client_shim,&format!("/withdraw/init"),
        &WithdrawMsg1 {
            shared_key_id: shared_key_id.clone(),
            state_chain_sig,
        })?;

    // Get state chain info
    let sc_info = get_statechain(&wallet.client_shim, &state_chain_id)?;
    // Get state entity withdraw fee info
    let se_fee_info = get_statechain_fee_info(&wallet.client_shim)?;

    // Construct withdraw tx
    let tx_withdraw_unsigned = tx_withdraw_build(
        &sc_info.utxo.txid,
        &rec_address,
        &(sc_info.amount+se_fee_info.deposit),
        &se_fee_info.withdraw,
        &se_fee_info.address
    )?;

    // co-sign withdraw tx
    let tx_w_prepare_sign_msg = PrepareSignTxMsg {
        protocol: Protocol::Withdraw,
        tx: tx_withdraw_unsigned.clone(),
        input_addrs: vec!(pk),
        input_amounts: vec!(sc_info.amount),
        proof_key: None,
    };
    cosign_tx_input(wallet, &shared_key_id, &tx_w_prepare_sign_msg)?;


    let witness: Vec<Vec<u8>> = requests::postb(&wallet.client_shim,&format!("/withdraw/confirm"),
        &WithdrawMsg2 {
            shared_key_id: shared_key_id.clone(),
            address: rec_address.to_string(),
        })?;

    let mut tx_withdraw_signed = tx_withdraw_unsigned.clone();
    tx_withdraw_signed.input[0].witness = witness;

    // Mark funds as withdrawn in wallet
    {
        let mut shared_key = wallet.get_shared_key_mut(shared_key_id)?;
        shared_key.unspent = false;
    }

    // Broadcast transcation
    let withdraw_txid = wallet.electrumx_client.broadcast_transaction(hex::encode(consensus::serialize(&tx_withdraw_signed)))?;
    debug!("Deposit: Funding tx broadcast. txid: {}", withdraw_txid);

    Ok((withdraw_txid, state_chain_id, state_chain_data.amount-se_fee_info.withdraw))
}
