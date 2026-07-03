use soroban_sdk::{contractclient, Address, BytesN, Env};

/// Cross-contract interface for Circle's TokenMessengerMinter contract on Stellar.
#[allow(dead_code)]
#[contractclient(name = "TokenMessengerMinterClient")]
pub trait TokenMessengerMinter {
    fn deposit_for_burn(
        e: &Env,
        caller: Address,
        amount: i128,
        destination_domain: u32,
        mint_recipient: BytesN<32>,
        burn_token: Address,
        destination_caller: BytesN<32>,
        max_fee: i128,
        min_finality_threshold: u32,
    );
}
