use near_sdk::{
    env::{self},
    log, near, require, AccountId, Gas, NearToken, PanicOnDefault, Promise,
};
use omni_transaction::signer::types::{SignRequest, SignatureResponse};

mod chainsig;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    pub mpc_contract_id: AccountId,
}

#[near]
impl Contract {
    #[init]
    pub fn new(mpc_contract_id: AccountId) -> Self {
        Self {
            mpc_contract_id, // v1.signer-prod.testnet for testnet v1.signer for mainnet
        }
    }

    pub fn request_signatures(&mut self, requests: Vec<SignRequest>) -> Promise {
        for (_, request) in requests.iter().enumerate() {
            require!(
                request.key_version == 0,
                "Key version must be 0, only secp256k1 is supported currently"
            );
        }

        chainsig::internal_request_signatures(requests, self.mpc_contract_id.clone())
    }

    #[private]
    pub fn resolve_signatures(
        &self,
        requests: Vec<SignRequest>,
    ) -> Vec<(SignRequest, Result<SignatureResponse, ()>)> {
        let mut results = Vec::new();
        let mut successful_count = 0;

        for (i, request) in requests.into_iter().enumerate() {
            let response = match env::promise_result(i as u64) {
                near_sdk::PromiseResult::Successful(data) => {
                    // Deserialize the SignatureResponse
                    match serde_json::from_slice::<SignatureResponse>(&data) {
                        Ok(sig_response) => {
                            successful_count += 1;
                            Ok(sig_response)
                        }
                        Err(e) => {
                            log!("Failed to deserialize signature response: {:?}", e);
                            Err(())
                        }
                    }
                }
                _ => {
                    log!("Signature request {} failed", i);
                    Err(())
                }
            };

            results.push((request, response));
        }

        let failed_count = results.len() - successful_count;
        log!(
            "Resolved {} signature results: {} successful, {} failed",
            results.len(),
            successful_count,
            failed_count
        );
        results
    }
}

// Delete account
// near account delete-account green-harbor.testnet beneficiary pivortex.testnet network-config testnet sign-with-legacy-keychain send

// Create account
// near account create-account sponsor-by-faucet-service green-harbor.testnet autogenerate-new-keypair save-to-legacy-keychain network-config testnet create

// Deploy
// cargo near deploy build-non-reproducible-wasm green-harbor.testnet with-init-call new json-args '{"mpc_contract_id": "v1.signer-prod.testnet"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-legacy-keychain send

// Works for up to 17 signatures
// Only supports secp256k1 currently