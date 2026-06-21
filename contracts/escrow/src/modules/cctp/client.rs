use soroban_sdk::{contractclient, Address, BytesN, Env};

/// Soroban client for Circle's `TokenMessengerMinter` contract on Stellar.
///
/// Interface source: https://github.com/circlefin/stellar-cctp
/// (`contracts/token-messenger-minter-v2`)
#[contractclient(name = "TokenMessengerMinterClient")]
pub trait TokenMessengerMinter {
    /// Burns USDC from `caller` and emits a cross-chain CCTP message.
    ///
    /// `caller` must authorize this call and approve `burn_token` spending beforehand.
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
