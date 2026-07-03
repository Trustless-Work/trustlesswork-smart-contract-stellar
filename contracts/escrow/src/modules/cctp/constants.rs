use soroban_sdk::{Address, Env};

pub const STELLAR_CCTP_DOMAIN: u32 = 27;
pub const CCTP_MIN_FINALITY_THRESHOLD_STANDARD: u32 = 2000;
pub const CCTP_DEFAULT_MAX_FEE: i128 = 0;
pub const STELLAR_TO_CCTP_DECIMAL_FACTOR: i128 = 10;

pub const CCTP_TOKEN_MESSENGER_STRKEY: &str =
    "CDNG7HXAPBWICI2E3AUBP3YZWZELJLYSB6F5CC7WLDTLTHVM74SLRTHP";

pub const VALID_CCTP_DESTINATION_DOMAINS: &[u32] = &[0, 1, 2, 3, 5, 6, 7];

#[inline]
pub fn cctp_token_messenger_address(e: &Env) -> Address {
    Address::from_str(e, CCTP_TOKEN_MESSENGER_STRKEY)
}

#[inline]
pub fn is_valid_cctp_destination_domain(domain: u32) -> bool {
    domain != STELLAR_CCTP_DOMAIN && VALID_CCTP_DESTINATION_DOMAINS.contains(&domain)
}
