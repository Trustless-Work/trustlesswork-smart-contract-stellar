use soroban_sdk::{contracttype, BytesN, Env};

/// Sentinel domain value meaning cross-chain release is disabled (standard Stellar payout).
pub const CROSS_CHAIN_DISABLED_DOMAIN: u32 = u32::MAX;

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct CrossChainReceiver {
    pub destination_domain: u32,
    pub recipient: BytesN<32>,
}

/// Returns `true` when escrow should release via CCTP instead of Stellar transfer.
#[inline]
pub fn is_cross_chain_configured(receiver: &CrossChainReceiver) -> bool {
    receiver.destination_domain != CROSS_CHAIN_DISABLED_DOMAIN
}

/// Default receiver — standard Stellar release (backwards compatible).
#[inline]
pub fn default_cross_chain_receiver(env: &Env) -> CrossChainReceiver {
    CrossChainReceiver {
        destination_domain: CROSS_CHAIN_DISABLED_DOMAIN,
        recipient: BytesN::from_array(env, &[0u8; 32]),
    }
}
