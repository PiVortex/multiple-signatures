use near_sdk::{
    env::{self},
    log, near, Gas, NearToken, Promise,
};
use sha2::{Digest, Sha256};

mod chainsig;

#[near(contract_state)]
pub struct Contract {}

impl Default for Contract {
    fn default() -> Self {
        Self {}
    }
}

#[near]
impl Contract {
    pub fn request_signatures(loops: u16, tgas_per_call: u16) {
        let key_type = "Ecdsa".to_string();
        let gas = Gas::from_tgas(tgas_per_call.into());

        for i in 0..loops {
            let path = format!("loop {}", i);

            let mut hasher = Sha256::new();
            let string_to_hash = format!("testing {}", i);
            hasher.update(string_to_hash.as_bytes());

            let payload = format!("{:x}", hasher.finalize());

            chainsig::internal_request_signature(path, payload, key_type.clone(), gas);
            log!("Loop: {}", i);
        }
    }
}

// Deploy
// cargo near deploy build-non-reproducible-wasm green-harbor.testnet without-init-call network-config testnet sign-with-legacy-keychain send

// Call
// near contract call-function as-transaction green-harbor.testnet request_signatures json-args '{"loops": 20, "tgas_per_call": 10}' prepaid-gas '300.0 Tgas' attached-deposit '0 NEAR' sign-as pivortex.testnet network-config testnet sign-with-legacy-keychain send
