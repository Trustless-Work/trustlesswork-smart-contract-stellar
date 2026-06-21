use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, BytesN, Env};

use crate::error::ContractError;
use crate::modules::cctp::client::TokenMessengerMinterClient;
use crate::modules::cctp::constants::{
    cctp_token_messenger_address, CCTP_DEFAULT_MAX_FEE, CCTP_MIN_FINALITY_THRESHOLD_STANDARD,
};
use crate::modules::cctp::decimal::{cctp_remainder, truncate_to_6_decimals};
use crate::storage::types::CrossChainReceiver;

const APPROVE_LEDGER_TTL: u32 = 100_000;

/// Pays the receiver net amount via CCTP burn, sending any 7th-decimal remainder to Stellar.
pub fn release_receiver_amount_via_cctp(
    e: &Env,
    token_client: &TokenClient,
    contract_address: &Address,
    burn_token: &Address,
    receiver_amount: i128,
    cross_chain: &CrossChainReceiver,
    stellar_receiver: &Address,
) -> Result<(), ContractError> {
    let token_messenger = cctp_token_messenger_address(e);
    release_receiver_amount_via_cctp_with_messenger(
        e,
        token_client,
        contract_address,
        &token_messenger,
        burn_token,
        receiver_amount,
        cross_chain,
        stellar_receiver,
    )
}

/// Same as [`release_receiver_amount_via_cctp`] but accepts an explicit TokenMessenger address (for tests).
pub fn release_receiver_amount_via_cctp_with_messenger(
    e: &Env,
    token_client: &TokenClient,
    contract_address: &Address,
    token_messenger: &Address,
    burn_token: &Address,
    receiver_amount: i128,
    cross_chain: &CrossChainReceiver,
    stellar_receiver: &Address,
) -> Result<(), ContractError> {
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
            &cross_chain.destination_domain,
            &cross_chain.recipient,
            burn_token,
            &destination_caller,
            &CCTP_DEFAULT_MAX_FEE,
            &CCTP_MIN_FINALITY_THRESHOLD_STANDARD,
        );
    }

    if remainder > 0 {
        token_client.transfer(contract_address, stellar_receiver, &remainder);
    }

    Ok(())
}
