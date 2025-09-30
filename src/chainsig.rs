use crate::*;
use near_sdk::ext_contract;

pub use ::mpc_contract::primitives::signature::SignRequestArgs;

#[allow(dead_code)]
#[ext_contract(mpc_contract_ext)]
trait MPCContract {
    fn sign(&self, request: SignRequestArgs);
}

const ATTACHED_DEPOSIT: NearToken = NearToken::from_yoctonear(1);
const GAS: Gas = Gas::from_tgas(15);

pub fn internal_request_signature(
    request: SignRequestArgs,
) -> Promise {
    let mpc_contract_id = if env::current_account_id().as_str().contains("testnet") {
        "v1.signer-prod.testnet"
    } else {
        "v1.signer"
    };

    mpc_contract_ext::ext(mpc_contract_id.parse().unwrap())
        .with_static_gas(GAS)
        .with_attached_deposit(ATTACHED_DEPOSIT)
        .sign(request)
}