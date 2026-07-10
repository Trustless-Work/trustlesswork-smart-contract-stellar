use soroban_sdk::{Address, Bytes, Env};

pub const STELLAR_CCTP_DOMAIN: u32 = 27;
pub const CCTP_MIN_FINALITY_THRESHOLD_STANDARD: u32 = 2000;
pub const CCTP_DEFAULT_MAX_FEE: i128 = 0;
pub const STELLAR_TO_CCTP_DECIMAL_FACTOR: i128 = 10;

/// Circle's Forwarding Service fee, expressed in Stellar's 7-decimal
/// stroops (same unit as the burn `amount`), deducted from the minted
/// amount on top of whatever CCTP protocol fee applies to the route. This
/// is the one part of `max_fee` we can't compute on-chain — Circle doesn't
/// expose it to a Soroban contract (no HTTP oracle), only via their REST
/// API (`GET /v2/burn/USDC/fees/{source}/{dest}?forward=true`, `forwardFee`
/// field). The protocol-fee portion is NOT guessed: it's queried live from
/// the messenger via `get_min_fee_amount` in `release.rs`.
///
/// CALIBRATED against a real testnet forward (2026-07-10, Stellar→Base
/// domain 6): a first attempt at 2_000_000 ($0.20, from Circle's blog post)
/// failed with `INSUFFICIENT_FEE` — Iris's live quote for that route was
/// 202_863–203_078 (6-decimal units, ~$0.203), not $0.20 flat. This constant
/// is set with a margin over that observed range. Circle's docs note the
/// forwarder consumes ~the full `max_fee` rather than refunding the unused
/// portion, so don't pad this much further than necessary — re-check
/// `GET /v2/burn/USDC/fees/27/{dest}?forward=true` if forwards start failing
/// with `INSUFFICIENT_FEE` again (the fee can drift over time).
pub const CCTP_FORWARDING_SERVICE_FEE_STROOPS: i128 = 2_500_000;

/// Reserved hook data that opts a burn into Circle's Forwarding Service:
/// 24 bytes of the ASCII magic `cctp-forward` (zero-padded to 24), a `u32`
/// version (`0`), and a `u32` length of any trailing custom payload (`0`,
/// none here). 32 bytes total. Verified against Circle's forwarding-service
/// docs and a third-party Stellar-source CCTP forwarding experiment
/// (`ElliotFriend/stunning-octo-carnival`, branch
/// `experiment/forwarder-stellar-source`) — get a single byte wrong here and
/// the burned funds are unrecoverable, so any change to this must be
/// re-verified against a real testnet burn before use.
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
