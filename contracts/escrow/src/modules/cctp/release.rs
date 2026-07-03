use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, BytesN, Env};

use crate::error::CctpError;
use crate::modules::cctp::client::TokenMessengerMinterClient;
use crate::modules::cctp::constants::{
    cctp_token_messenger_address, is_valid_cctp_destination_domain, CCTP_DEFAULT_MAX_FEE,
    CCTP_MIN_FINALITY_THRESHOLD_STANDARD,
};
use crate::modules::cctp::decimal::{cctp_remainder, truncate_to_6_decimals};

const APPROVE_LEDGER_TTL: u32 = 100_000;

/// Validates a destination before it is stored, so a burn at release never fails.
pub fn validate_destination(
    destination_domain: u32,
    mint_recipient: &BytesN<32>,
) -> Result<(), CctpError> {
    if !is_valid_cctp_destination_domain(destination_domain) {
        return Err(CctpError::InvalidDestinationDomain);
    }
    if mint_recipient.to_array() == [0u8; 32] {
        return Err(CctpError::InvalidRecipient);
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
