extern crate std;

use crate::contract::EscrowContract;
use crate::contract::EscrowContractClient;
use soroban_sdk::{token, Address, Env};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

pub fn create_usdc_token<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    (
        TokenClient::new(e, &sac.address()),
        TokenAdminClient::new(e, &sac.address()),
    )
}

pub struct TestData<'a> {
    pub client: EscrowContractClient<'a>,
}

pub fn create_escrow_contract<'a>(env: &Env) -> TestData<'a> {
    env.mock_all_auths();
    let client = EscrowContractClient::new(env, &env.register(EscrowContract {}, ()));
    TestData { client }
}
