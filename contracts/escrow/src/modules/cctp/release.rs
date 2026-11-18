use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, BytesN, Env};

use crate::error::CctpError;
use crate::modules::cctp::client::TokenMessengerMinterClient;
use crate::modules::cctp::constants::{
    cctp_forward_hook_data, cctp_token_messenger_address, is_valid_cctp_destination_domain,
    CCTP_DEFAULT_MAX_FEE, CCTP_MIN_FINALITY_THRESHOLD_STANDARD,
};
use crate::modules::cctp::decimal::{cctp_remainder, truncate_to_6_decimals};

const APPROVE_LEDGER_TTL: u32 = 100_000;

/// Max share of the route's amount `max_fee` is allowed to claim — 10%.
/// Defense-in-depth only: the API computes the real value from a live
/// Circle quote (typically a small fraction of a percent), this just bounds
/// how much a bogus/compromised value could claim if the entrypoint is
/// called directly, bypassing the API.
const MAX_FEE_CAP_DIVISOR: i128 = 10;

/// Validates a destination before it is stored, so a burn at release never
/// fails. `route_amount` is the escrow amount (single-release) or the
/// milestone amount (multi-release) `max_fee` is bounded against.
pub fn validate_destination(
    destination_domain: u32,
    mint_recipient: &BytesN<32>,
    max_fee: i128,
    route_amount: i128,
) -> Result<(), CctpError> {
    if !is_valid_cctp_destination_domain(destination_domain) {
        return Err(CctpError::InvalidDestinationDomain);
    }
    if mint_recipient.to_array() == [0u8; 32] {
        return Err(CctpError::InvalidRecipient);
    }
    if max_fee < 0 || max_fee > route_amount / MAX_FEE_CAP_DIVISOR {
        return Err(CctpError::MaxFeeExceedsCap);
    }
    Ok(())
}

/// Burns the receiver's share via CCTP; any 7th-decimal remainder goes to the
/// Stellar receiver so nothing stays locked.
#[allow(clippy::too_many_arguments)]
pub fn release_receiver_amount_via_cctp(
    e: &Env,
    token_client: &TokenClient,
    contract_address: &Address,
    burn_token: &Address,
    receiver_amount: i128,
    destination_domain: u32,
    mint_recipient: &BytesN<32>,
    stellar_receiver: &Address,
) {
    let token_messenger = cctp_token_messenger_address(e);
    release_receiver_amount_via_cctp_with_messenger(
        e,
        token_client,
        contract_address,
        &token_messenger,
        burn_token,
        receiver_amount,
        destination_domain,
        mint_recipient,
        stellar_receiver,
    );
}

/// Same as above but with an explicit TokenMessenger address (for tests).
#[allow(clippy::too_many_arguments)]
pub fn release_receiver_amount_via_cctp_with_messenger(
    e: &Env,
    token_client: &TokenClient,
    contract_address: &Address,
    token_messenger: &Address,
    burn_token: &Address,
    receiver_amount: i128,
    destination_domain: u32,
    mint_recipient: &BytesN<32>,
    stellar_receiver: &Address,
) {
    let burn_amount = truncate_to_6_decimals(receiver_amount);
    let remainder = cctp_remainder(receiver_amount);

    if burn_amount > 0 {
        let expiration_ledger = e.ledger().sequence() + APPROVE_LEDGER_TTL;
        token_client.approve(
            contract_address,
            token_messenger,
            &burn_amount,
            &expiration_ledger,
        );

        let destination_caller = BytesN::from_array(e, &[0u8; 32]);
        let messenger_client = TokenMessengerMinterClient::new(e, token_messenger);
        messenger_client.deposit_for_burn(
            contract_address,
            &burn_amount,
            &destination_domain,
            mint_recipient,
            burn_token,
            &destination_caller,
            &CCTP_DEFAULT_MAX_FEE,
            &CCTP_MIN_FINALITY_THRESHOLD_STANDARD,
        );
    }

    if remainder > 0 {
        token_client.transfer(contract_address, stellar_receiver, &remainder);
    }
}

/// Same as `release_receiver_amount_via_cctp`, but opts the burn into
/// Circle's Forwarding Service: the destination-chain mint completes
/// automatically (no second, EVM-side signature from the receiver). Circle
/// deducts `max_fee` (protocol fee, queried live via `get_min_fee_amount`,
/// plus `approved_max_fee` — the forwarding-service ceiling the receiver
/// signed off on in `set_cross_chain_destination`, sized by the API from a
/// live Circle quote at that time) from the minted amount — the forwarder
/// consumes ~the full `max_fee`, it isn't a cap with a refund of the unused
/// portion.
///
/// `deposit_for_burn_with_hook`'s parameter list and `get_min_fee_amount` were
/// confirmed against the real deployed `TokenMessengerMinter` on testnet via
/// `stellar contract info interface`. Verified end-to-end on testnet
/// (2026-07-10, Stellar→Base): `forwardTxHash` came back in the Iris
/// attestation with no receiver-side EVM signature.
#[allow(clippy::too_many_arguments)]
pub fn release_receiver_amount_via_cctp_forwarding(
    e: &Env,
    token_client: &TokenClient,
    contract_address: &Address,
    burn_token: &Address,
    receiver_amount: i128,
    destination_domain: u32,
    mint_recipient: &BytesN<32>,
    approved_max_fee: i128,
    stellar_receiver: &Address,
) {
    let token_messenger = cctp_token_messenger_address(e);
    release_receiver_amount_via_cctp_forwarding_with_messenger(
        e,
        token_client,
        contract_address,
        &token_messenger,
        burn_token,
        receiver_amount,
        destination_domain,
        mint_recipient,
        approved_max_fee,
        stellar_receiver,
    );
}

/// Same as above but with an explicit TokenMessenger address (for tests).
#[allow(clippy::too_many_arguments)]
pub fn release_receiver_amount_via_cctp_forwarding_with_messenger(
    e: &Env,
    token_client: &TokenClient,
    contract_address: &Address,
    token_messenger: &Address,
    burn_token: &Address,
    receiver_amount: i128,
    destination_domain: u32,
    mint_recipient: &BytesN<32>,
    approved_max_fee: i128,
    stellar_receiver: &Address,
) {
    let burn_amount = truncate_to_6_decimals(receiver_amount);
    let remainder = cctp_remainder(receiver_amount);

    if burn_amount > 0 {
        let expiration_ledger = e.ledger().sequence() + APPROVE_LEDGER_TTL;
        let messenger_client = TokenMessengerMinterClient::new(e, token_messenger);

        // Protocol fee is queried live, not guessed. `approved_max_fee` is
        // the receiver-approved forwarding-service ceiling, sized by the API
        // from a live Circle quote when the destination was registered —
        // NOT recomputed here, so it can't drift stale between set and
        // release. `max_fee` is deducted from `burn_amount` at mint time on
        // the destination chain, not pulled as an extra amount from the
        // source — so the approved allowance stays `burn_amount`.
        let protocol_fee = messenger_client.get_min_fee_amount(burn_token, &burn_amount);
        let max_fee = protocol_fee + approved_max_fee;

        token_client.approve(
            contract_address,
            token_messenger,
            &burn_amount,
            &expiration_ledger,
        );

        // Zero is required here — a nonzero `destination_caller` disables
        // forwarding per Circle's docs.
        let destination_caller = BytesN::from_array(e, &[0u8; 32]);
        messenger_client.deposit_for_burn_with_hook(
            contract_address,
            &burn_amount,
            &destination_domain,
            mint_recipient,
            burn_token,
            &destination_caller,
            &max_fee,
            &CCTP_MIN_FINALITY_THRESHOLD_STANDARD,
            &cctp_forward_hook_data(e),
        );
    }

    if remainder > 0 {
        token_client.transfer(contract_address, stellar_receiver, &remainder);
    }
}
