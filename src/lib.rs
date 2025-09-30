use ::mpc_contract::{crypto_shared::SignatureResponse, primitives::signature::SignRequestArgs};
use near_sdk::{
    env::{self},
    json_types::U64,
    log, near,
    store::LookupMap,
    BorshStorageKey, Gas, NearToken, Promise, PromiseError,
};

mod chainsig;
mod types;

use types::{
    sign_request_args_to_sign_request, RequestAndResponse, StoredRequestAndResponse,
    StoredSignatureResponse,
};

#[derive(BorshStorageKey)]
#[near]
pub enum StorageKey {
    RequestsAndResponses,
}

#[near(contract_state)]
pub struct Contract {
    pub requests_and_responses: LookupMap<u64, Vec<StoredRequestAndResponse>>,
    pub batched_request_index: u64,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            requests_and_responses: LookupMap::new(StorageKey::RequestsAndResponses),
            batched_request_index: 0,
        }
    }
}

#[near]
impl Contract {
    pub fn request_signatures(&mut self, requests: Vec<SignRequestArgs>) {
        // Get the current batched request index and increment it
        let current_batched_request_index = self.batched_request_index;
        self.batched_request_index = self.batched_request_index + 1;

        let number_of_requests_in_batch = U64(requests.len() as u64);

        for (inner_request_num, request) in requests.iter().enumerate() {
            // Make the signature request
            chainsig::internal_request_signature(
                request.clone(),
                U64(current_batched_request_index),
                number_of_requests_in_batch,
            );
            log!(
                "Request {} of batched request {}: {:?}",
                inner_request_num,
                current_batched_request_index,
                request
            );
        }
    }

    #[private]
    pub fn signature_callback(
        &mut self,
        #[callback_result] call_result: Result<SignatureResponse, PromiseError>,
        request: SignRequestArgs,
        current_batched_request_index: U64,
        number_of_requests_in_batch: U64,
    ) -> Option<Vec<RequestAndResponse>> {
        // Convert SignRequestArgs to SignRequest
        let sign_request = sign_request_args_to_sign_request(request);

        // Handle the callback result (success or error)
        let stored_response = match call_result {
            Ok(response) => StoredSignatureResponse::from_signature_response(response),
            Err(promise_error) => {
                log!("A signature request failed: {:?}", promise_error);
                Err(())
            }
        };

        // Create the completed request and response
        let request_and_response = StoredRequestAndResponse {
            request: sign_request,
            response: stored_response,
        };

        // Get or create the batch vector
        let mut batch_responses = self
            .requests_and_responses
            .get(&current_batched_request_index.0)
            .cloned()
            .unwrap_or_else(|| Vec::new());

        // Add the new response to the batch
        batch_responses.push(request_and_response);

        // Check if batch is complete by comparing current length to expected total
        let current_count = batch_responses.len() as u64;

        log!(
            "Batch {}: {}/{} completed",
            current_batched_request_index.0,
            current_count,
            number_of_requests_in_batch.0
        );

        // Check if batch is complete
        if current_count >= number_of_requests_in_batch.0 {
            // Batch is complete! Convert to API format and clean up
            let responses: Vec<RequestAndResponse> = batch_responses
                .into_iter()
                .map(|stored| RequestAndResponse::from_stored(stored))
                .collect();

            // Remove from storage
            self.requests_and_responses
                .remove(&current_batched_request_index.0);

            log!(
                "Batch {} completed! Returning {} responses",
                current_batched_request_index.0,
                responses.len()
            );

            return Some(responses);
        } else {
            // Batch not complete yet, just store the updated batch
            self.requests_and_responses
                .insert(current_batched_request_index.0, batch_responses);
            return None;
        }
    }
}

// Deploy
// cargo near deploy build-non-reproducible-wasm green-harbor.testnet without-init-call network-config testnet sign-with-legacy-keychain send
