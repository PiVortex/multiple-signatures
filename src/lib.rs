use near_sdk::{
    env::{self},
    log, near, Gas, NearToken, Promise,
};

mod chainsig;
use chainsig::SignRequestArgs;

#[near(contract_state)]
pub struct Contract {}

impl Default for Contract {
    fn default() -> Self {
        Self {}
    }
}

#[near]
impl Contract {
    pub fn request_signatures(&mut self, requests: Vec<SignRequestArgs>) {
        
        for (index, request) in requests.iter().enumerate() {
            chainsig::internal_request_signature(request.clone());
            log!("Request {}: {:?}", index, request);
        }
    }
}

// Deploy
// cargo near deploy build-non-reproducible-wasm green-harbor.testnet without-init-call network-config testnet sign-with-legacy-keychain send
