extern crate std;

use crate::contract::EscrowContract;
use crate::contract::EscrowContractClient;
use crate::storage::types::{CrossChainDestination, Receiver};
use soroban_sdk::{token, Address, BytesN, Env};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

pub fn create_usdc_token<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    (
        TokenClient::new(e, &sac.address()),
        TokenAdminClient::new(e, &sac.address()),
    )
}

/// A receiver with a Stellar auth address and a valid CCTP destination
/// (Ethereum, non-zero recipient, zero max_fee) for tests.
pub fn test_receiver(env: &Env, auth: &Address) -> Receiver {
    let mut recipient = [0u8; 32];
    recipient[31] = 7;
    Receiver {
        stellar_address: Some(auth.clone()),
        cctp: CrossChainDestination {
            destination_domain: 0,
            mint_recipient: BytesN::from_array(env, &recipient),
        },
    }
}

/// Registers the CCTP TokenMessenger mock at its well-known address, so a
/// release (which always burns cross-chain) has somewhere to burn to.
/// Returns the messenger address for balance assertions.
pub fn register_mock_token_messenger(env: &Env) -> Address {
    let messenger = Address::from_str(
        env,
        crate::modules::cctp::constants::CCTP_TOKEN_MESSENGER_STRKEY,
    );
    env.register_at(&messenger, crate::tests::cctp::MockTokenMessenger, ());
    messenger
}

pub struct TestData<'a> {
    pub client: EscrowContractClient<'a>,
}

pub fn create_escrow_contract<'a>(env: &Env, admin: &Address) -> TestData<'a> {
    env.mock_all_auths();
    let wasm_hash = BytesN::from_array(env, &[0u8; 32]);
    let client =
        EscrowContractClient::new(env, &env.register(EscrowContract {}, (admin, &wasm_hash)));
    TestData { client }
}
