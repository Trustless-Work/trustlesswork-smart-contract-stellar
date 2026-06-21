use soroban_sdk::BytesN;

use crate::error::ContractError;
use crate::modules::cctp::constants::is_valid_cctp_destination_domain;
use crate::storage::types::{is_cross_chain_configured, CrossChainReceiver};

#[inline]
pub fn is_zero_bytes32(bytes: &BytesN<32>) -> bool {
    bytes.to_array() == [0u8; 32]
}

#[inline]
pub fn validate_cross_chain_receiver(
    receiver: &CrossChainReceiver,
) -> Result<(), ContractError> {
    if !is_cross_chain_configured(receiver) {
        return Ok(());
    }

    if !is_valid_cctp_destination_domain(receiver.destination_domain) {
        return Err(ContractError::InvalidCctpDestinationDomain);
    }

    if is_zero_bytes32(&receiver.recipient) {
        return Err(ContractError::InvalidCctpRecipient);
    }

    Ok(())
}
