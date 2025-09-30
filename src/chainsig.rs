use crate::*;
use near_sdk::ext_contract;

#[allow(dead_code)]
#[ext_contract(mpc_contract_ext)]
trait MPCContract {
    fn sign(&self, request: SignRequestArgs);
}

const ATTACHED_DEPOSIT: NearToken = NearToken::from_yoctonear(1);
const SIGNATURE_GAS: Gas = Gas::from_tgas(15);
const CALLBACK_GAS: Gas = Gas::from_tgas(15);

pub fn internal_request_signature(
    request: SignRequestArgs,
    current_batched_request_index: U64,
    number_of_requests_in_batch: U64,
) -> Promise {
    let mpc_contract_id = if env::current_account_id().as_str().contains("testnet") {
        "v1.signer-prod.testnet"
    } else {
        "v1.signer"
    };

    mpc_contract_ext::ext(mpc_contract_id.parse().unwrap())
        .with_static_gas(SIGNATURE_GAS)
        .with_attached_deposit(ATTACHED_DEPOSIT)
        .sign(request.clone())
        .then(
            crate::Contract::ext(env::current_account_id())
                .with_static_gas(CALLBACK_GAS)
                .signature_callback(
                    request,
                    current_batched_request_index,
                    number_of_requests_in_batch,
                ),
        )
}
