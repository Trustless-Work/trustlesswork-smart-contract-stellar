use soroban_sdk::{Address, Bytes, Env};

pub const STELLAR_CCTP_DOMAIN: u32 = 27;
pub const CCTP_MIN_FINALITY_THRESHOLD_STANDARD: u32 = 2000;
pub const STELLAR_TO_CCTP_DECIMAL_FACTOR: i128 = 10;

/// Reserved hook data that opts a burn into Circle's Forwarding Service:
/// `cctp-forward` zero-padded to 24 bytes, version `0` and payload length `0`
/// (both u32), 32 bytes total. A wrong byte means Circle won't forward the mint.
#[inline]
pub fn cctp_forward_hook_data(e: &Env) -> Bytes {
    let mut bytes = [0u8; 32];
    bytes[0..12].copy_from_slice(b"cctp-forward");
    // bytes[12..24] stay zero (padding for the reserved 24-byte magic field).
    // bytes[24..28] stay zero (version = 0).
    // bytes[28..32] stay zero (trailing payload length = 0).
    Bytes::from_array(e, &bytes)
}

/// Circle's TokenMessengerMinter on mainnet.
pub const CCTP_TOKEN_MESSENGER_STRKEY: &str =
    "CAE2G5Z77UP7GYPYGFOWFGW7C7J6I4YP2AFGSADRKQY62SYUFLPNFTXL";

pub const VALID_CCTP_DESTINATION_DOMAINS: &[u32] = &[0, 1, 2, 3, 5, 6, 7];

#[inline]
pub fn cctp_token_messenger_address(e: &Env) -> Address {
    Address::from_str(e, CCTP_TOKEN_MESSENGER_STRKEY)
}

#[inline]
pub fn is_valid_cctp_destination_domain(domain: u32) -> bool {
    domain != STELLAR_CCTP_DOMAIN && VALID_CCTP_DESTINATION_DOMAINS.contains(&domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Circle's documented forward-only hook (v0): 0x636374702d666f7277617264 + 20 zero bytes.
    const CIRCLE_FORWARD_HOOK_V0: [u8; 32] = [
        0x63, 0x63, 0x74, 0x70, 0x2d, 0x66, 0x6f, 0x72, 0x77, 0x61, 0x72, 0x64, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[test]
    fn forward_hook_matches_circle_documented_bytes() {
        let e = Env::default();
        assert_eq!(
            cctp_forward_hook_data(&e),
            Bytes::from_array(&e, &CIRCLE_FORWARD_HOOK_V0)
        );
    }
}
