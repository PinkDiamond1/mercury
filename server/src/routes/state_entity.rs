//! state_entity
//!
//! State Entity implementation

use kms::ecdsa::two_party::MasterKey1;
use super::super::Result;
use rocket_contrib::json::Json;
use rocket::State;

use crate::storage::mock::{ StateChain, MockStorage };
use crate::routes::ecdsa::EcdsaStruct;

use super::super::auth::jwt::Claims;
use super::super::storage::db;
use super::super::Config;

/// State struct representing an active UTXO shared by state entity and Owner
#[allow(dead_code)]
pub struct UserState {
    id: u32,
    utxo: String,
    key: String,
    state_chain: StateChain
    // owner_auth:
}
/// State Entity main
pub struct StateEntity {
    /// storage
    pub storage: MockStorage
}

// deposit() messages:
// 0. 2P-ECDSA to gen shared key P
// 1. user sends funding tx outpoint, B1, C1
// 2. Co-op sign kick-off tx (generated by SE)
// 3. Co-op sign back-up tx (generated by user)
#[allow(dead_code)]
#[derive(Serialize, Deserialize,Debug)]
pub struct DepositMessage1 {
    txid: String,
    vout: u32,
    backup_pubkey: String,
    proof_pubkey: String
}
#[post("/deposit/<id>/first", format = "json", data = "<deposit_first_msg>")]
pub fn deposit_first(
    state: State<Config>,
    claim: Claims,
    id: String,
    deposit_first_msg: Json<DepositMessage1>,
) -> Result<Json<(String,String)>>{

    // master key stored in EcdsaStruct due to 2P-ECDSA implementation.
    // Retrieve from here for now
    let master_key: MasterKey1 = db::get(&state.db, &claim.sub, &id, &EcdsaStruct::Party1MasterKey)?
        .ok_or(format_err!("No data for such identifier {}", id))?;

        println!("mk public: {:?}",master_key.public);
        println!("deposit_first_msg: {:?}",deposit_first_msg);

    Ok(Json((String::from("deposit"),String::from("first"))))
}





#[cfg(test)]
mod tests {
    // Mock Owner can be used to implement and test client side of protocol
    use bitcoin::util;
    use crate::util::generate_keypair;
    /// public/private key pairs.
    #[allow(dead_code)]
    pub struct KeyPair {
        priv_key: util::key::PrivateKey,
        pub_key: util::key::PublicKey
    }
    impl KeyPair {
        /// generate random key pair
        pub fn new() -> Self {
            let key_pair = generate_keypair();
            KeyPair{ priv_key: key_pair.0, pub_key: key_pair.1 }
        }
    }
    /// Rpc implementation of Owner
    #[allow(dead_code)]
    pub struct MockOwner {
        /// Rpc client instance
        id: u32,
        /// Keys
        keys: Vec<KeyPair>
    }
    impl MockOwner {
        /// Init with single key
        pub fn new() -> Self {
            MockOwner{ id: 1, keys: vec![KeyPair::new()] }
        }
        /// generate new key and add to self.keys
        pub fn new_key(&mut self) {
            let key_pair = generate_keypair();
            self.keys.push(KeyPair{ priv_key: key_pair.0, pub_key: key_pair.1 })
        }
    }


}
