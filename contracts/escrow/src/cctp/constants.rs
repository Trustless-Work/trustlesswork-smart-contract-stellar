/// Stellar CCTP domain ID (assigned by Circle)
#[allow(dead_code)]
pub const STELLAR_CCTP_DOMAIN: u32 = 27;

/// CCTP domain IDs for supported destination chains
pub const CCTP_DOMAIN_ETHEREUM: u32 = 0;
pub const CCTP_DOMAIN_AVALANCHE: u32 = 1;
pub const CCTP_DOMAIN_OP_MAINNET: u32 = 2;
pub const CCTP_DOMAIN_ARBITRUM_ONE: u32 = 3;
pub const CCTP_DOMAIN_SOLANA: u32 = 5;
pub const CCTP_DOMAIN_BASE: u32 = 6;
pub const CCTP_DOMAIN_POLYGON_POS: u32 = 7;

/// USDC decimal difference between Stellar (7) and other chains (6)
/// To convert: amount_6_decimals = amount_7_decimals / USDC_DECIMAL_DIFF
pub const USDC_DECIMAL_DIFF: i128 = 10;

/// Testnet: TokenMessengerMinter contract address
/// From Circle's official docs:
/// https://developers.circle.com/cctp/references/stellar-contracts
pub const CCTP_TOKEN_MESSENGER_ADDRESS: &str =
    "CDNG7HXAPBWICI2E3AUBP3YZWZELJLYSB6F5CC7WLDTLTHVM74SLRTHP";

/// Testnet: MessageTransmitter contract address
/// From Circle's official docs:
/// https://developers.circle.com/cctp/references/stellar-contracts
#[allow(dead_code)]
pub const CCTP_MESSAGE_TRANSMITTER_ADDRESS: &str =
    "CBJ6MTCKKZG73PMDZCJMSFRD7DQEMI4FKDH7CGDSV4W6FHCRBCQAVVJY";

/// Minimum valid CCTP domain ID
#[allow(dead_code)]
pub const MIN_VALID_CCTP_DOMAIN: u32 = 0;

/// Maximum valid CCTP domain ID (as of current supported chains)
#[allow(dead_code)]
pub const MAX_VALID_CCTP_DOMAIN: u32 = 7;

/// Valid CCTP domain IDs for validation
pub const VALID_CCTP_DOMAINS: [u32; 7] = [
    CCTP_DOMAIN_ETHEREUM,
    CCTP_DOMAIN_AVALANCHE,
    CCTP_DOMAIN_OP_MAINNET,
    CCTP_DOMAIN_ARBITRUM_ONE,
    CCTP_DOMAIN_SOLANA,
    CCTP_DOMAIN_BASE,
    CCTP_DOMAIN_POLYGON_POS,
];

/// Check if a domain ID is a valid CCTP domain
#[inline]
pub fn is_valid_cctp_domain(domain: u32) -> bool {
    VALID_CCTP_DOMAINS.contains(&domain)
}

/// Truncate a Stellar USDC amount (7 decimals) to CCTP amount (6 decimals)
/// Returns (amount_6_decimals, remainder_7th_decimal)
#[inline]
pub fn truncate_to_6_decimals(amount_7_decimals: i128) -> (i128, i128) {
    let amount_6 = amount_7_decimals / USDC_DECIMAL_DIFF;
    let remainder = amount_7_decimals % USDC_DECIMAL_DIFF;
    (amount_6, remainder)
}
