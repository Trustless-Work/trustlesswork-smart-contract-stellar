use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env};

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

    /// Same as `deposit_for_burn`, but attaches `hook_data`. Circle's
    /// Forwarding Service watches for the reserved `cctp-forward` hook data
    /// (see `constants::cctp_forward_hook_data`) and, when present,
    /// automatically completes the mint on the destination chain — no
    /// second signature from the receiver.
    fn deposit_for_burn_with_hook(
        e: &Env,
        caller: Address,
        amount: i128,
        destination_domain: u32,
        mint_recipient: BytesN<32>,
        burn_token: Address,
        destination_caller: BytesN<32>,
        max_fee: i128,
        min_finality_threshold: u32,
        hook_data: Bytes,
    );

    /// Computes the exact CCTP protocol fee for burning `amount` of
    /// `burn_token`, per Circle's own `min_fee`/`min_fee_amount` config for
    /// that token. Used to size `max_fee` precisely instead of guessing.
    fn get_min_fee_amount(e: &Env, burn_token: Address, amount: i128) -> i128;
}
