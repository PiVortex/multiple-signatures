use crate::*;
use near_sdk::ext_contract;
use omni_transaction::signer::types::SignRequest;

#[allow(dead_code)]
#[ext_contract(mpc_contract_ext)]
trait MPCContract {
    fn sign(&self, request: SignRequest);
}

const ATTACHED_DEPOSIT: NearToken = NearToken::from_yoctonear(1);
const SIGNATURE_GAS: Gas = Gas::from_tgas(15);
const CALLBACK_GAS: Gas = Gas::from_tgas(15);

fn join_all<I>(mut iter: I) -> Promise
where
    I: Iterator<Item = Promise>,
{
    let first = iter.next().expect("Must have at least one promise");
    iter.fold(first, |acc, p| acc.and(p))
}

pub fn internal_request_signatures(
    requests: Vec<SignRequest>,
    mpc_contract_id: AccountId,
) -> Promise {
    // Create promises for all signature requests
    let calls = requests.iter().map(|request| {
        mpc_contract_ext::ext(mpc_contract_id.clone())
            .with_static_gas(SIGNATURE_GAS)
            .with_attached_deposit(ATTACHED_DEPOSIT)
            .sign(request.clone())
    });

    // Join all promises and then resolve with callback
    join_all(calls).then(
        crate::Contract::ext(env::current_account_id())
            .with_static_gas(CALLBACK_GAS)
            .resolve_signatures(requests),
    )
}
