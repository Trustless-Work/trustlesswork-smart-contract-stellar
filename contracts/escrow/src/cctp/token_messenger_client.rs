use soroban_sdk::{Address, Bytes, Env, IntoVal, Symbol, Val, Vec};

/// Client for Circle's TokenMessengerMinter contract on Stellar
/// Calls deposit_for_burn to initiate a cross-chain USDC transfer
pub struct TokenMessengerClient(Address);

impl TokenMessengerClient {
    pub fn new(_env: &Env, address: &Address) -> Self {
        Self(address.clone())
    }

    /// Call deposit_for_burn on the TokenMessengerMinter contract
    /// Burns USDC on Stellar and emits a message for minting on the destination chain
    pub fn deposit_for_burn(
        &self,
        env: &Env,
        amount: i128,
        destination_domain: u32,
        mint_recipient: Bytes,
        token_address: Address,
    ) {
        let args: Vec<Val> = (amount, destination_domain, mint_recipient, token_address)
            .into_val(env);
        env.invoke_contract::<Val>(&self.0, &Symbol::new(env, "deposit_for_burn"), args);
    }
}
