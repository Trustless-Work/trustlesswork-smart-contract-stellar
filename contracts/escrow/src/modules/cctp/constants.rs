use soroban_sdk::{Address, Bytes, Env};

pub const STELLAR_CCTP_DOMAIN: u32 = 27;
pub const CCTP_MIN_FINALITY_THRESHOLD_STANDARD: u32 = 2000;
pub const STELLAR_TO_CCTP_DECIMAL_FACTOR: i128 = 10;

/// Reserved hook data that opts a burn into Circle's Forwarding Service:
/// 24 bytes of the ASCII magic `cctp-forward` (zero-padded), a `u32` version
/// (`0`), and a `u32` trailing-payload length (`0`). 32 bytes total. A wrong
/// byte here makes the burned funds unrecoverable — re-verify any change
/// against a real testnet burn.
#[inline]
pub fn cctp_forward_hook_data(e: &Env) -> Bytes {
    let mut bytes = [0u8; 32];
    bytes[0..12].copy_from_slice(b"cctp-forward");
    // bytes[12..24] stay zero (padding for the reserved 24-byte magic field).
    // bytes[24..28] stay zero (version = 0).
    // bytes[28..32] stay zero (trailing payload length = 0).
    Bytes::from_array(e, &bytes)
}

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
