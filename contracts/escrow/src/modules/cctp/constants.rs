use soroban_sdk::{Address, Env};

/// Stellar CCTP domain identifier (source chain).
pub const STELLAR_CCTP_DOMAIN: u32 = 27;

/// Standard finality threshold for CCTP burn messages (Circle default).
pub const CCTP_MIN_FINALITY_THRESHOLD_STANDARD: u32 = 2000;

/// Default max fee passed to `deposit_for_burn` (local token decimals).
/// Zero means no fee budget; TokenMessenger validates against configured min fees.
pub const CCTP_DEFAULT_MAX_FEE: i128 = 0;

/// Factor to truncate Stellar USDC (7 decimals) to CCTP burn units (6 decimals).
pub const STELLAR_TO_CCTP_DECIMAL_FACTOR: i128 = 10;

/// Circle `TokenMessengerMinter` on Stellar Testnet (strkey).
/// See: https://developers.circle.com/cctp/references/stellar-contracts
pub const CCTP_TOKEN_MESSENGER_STRKEY: &str =
    "CDNG7HXAPBWICI2E3AUBP3YZWZELJLYSB6F5CC7WLDTLTHVM74SLRTHP";

/// Circle `MessageTransmitter` on Stellar Testnet (reference; outbound burns use TokenMessenger).
pub const CCTP_MESSAGE_TRANSMITTER_STRKEY: &str =
    "CBJ6MTCKKZG73PMDZCJMSFRD7DQEMI4FKDH7CGDSV4W6FHCRBCQAVVJY";

/// Known CCTP destination domains supported for outbound burns from Stellar.
pub const VALID_CCTP_DESTINATION_DOMAINS: &[u32] = &[0, 1, 2, 3, 5, 6, 7];

/// Resolves the Circle TokenMessengerMinter contract address on Stellar Testnet.
#[inline]
pub fn cctp_token_messenger_address(e: &Env) -> Address {
    Address::from_str(e, CCTP_TOKEN_MESSENGER_STRKEY)
}

/// Resolves the Circle MessageTransmitter contract address on Stellar Testnet.
#[inline]
pub fn cctp_message_transmitter_address(e: &Env) -> Address {
    Address::from_str(e, CCTP_MESSAGE_TRANSMITTER_STRKEY)
}

/// Returns true if `domain` is a supported outbound CCTP destination (not Stellar).
#[inline]
pub fn is_valid_cctp_destination_domain(domain: u32) -> bool {
    domain != STELLAR_CCTP_DOMAIN && VALID_CCTP_DESTINATION_DOMAINS.contains(&domain)
}
