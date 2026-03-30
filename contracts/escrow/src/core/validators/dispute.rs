use soroban_sdk::{Address, Map};

use crate::{
    error::ContractError,
    storage::types::Escrow,
};

const MAX_DISTRIBUTIONS: u32 = 50;

#[inline]
pub fn validate_withdraw_remaining_funds_conditions(
    escrow: &Escrow,
    dispute_resolver: &Address,
    all_processed: bool,
    current_balance: i128,
    total: i128,
    distributions: &Map<Address, i128>
) -> Result<(), ContractError> {
    if distributions.len() > MAX_DISTRIBUTIONS {
        return Err(ContractError::TooManyDistributions);
    }

    if !escrow.roles.dispute_resolvers.contains(dispute_resolver) {
        return Err(ContractError::OnlyDisputeResolverCanExecuteThisFunction);
    }

    if !all_processed {
        return Err(ContractError::EscrowNotFullyProcessed);
    }

    if total <= 0 {
        return Err(ContractError::TotalAmountCannotBeZero);
    }

    if current_balance < total {
        return Err(ContractError::InsufficientFundsForResolution);
    }

    Ok(())
}

#[inline]
pub fn validate_dispute_resolution_conditions(
    escrow: &Escrow,
    dispute_resolver: &Address,
    current_balance: i128,
    total: i128,
    distributions: &Map<Address, i128>,
) -> Result<(), ContractError> {
    if distributions.len() > MAX_DISTRIBUTIONS {
        return Err(ContractError::TooManyDistributions);
    }

    if !escrow.roles.dispute_resolvers.contains(dispute_resolver) {
        return Err(ContractError::OnlyDisputeResolverCanExecuteThisFunction);
    }

    if !escrow.flags.disputed {
        return Err(ContractError::EscrowNotInDispute);
    }

    if current_balance < total {
        return Err(ContractError::InsufficientFundsForResolution);
    }

    if total != current_balance {
        return Err(ContractError::DistributionsMustEqualEscrowBalance);
    }

    if total <= 0 {
        return Err(ContractError::TotalAmountCannotBeZero);
    }

    Ok(())
}

#[inline]
pub fn validate_dispute_flag_change_conditions(
    escrow: &Escrow,
    signer: &Address,
) -> Result<(), ContractError> {
    if escrow.flags.disputed {
        return Err(ContractError::EscrowAlreadyInDispute);
    }

    if escrow.flags.resolved {
        return Err(ContractError::EscrowAlreadyResolved);
    }

    if escrow.roles.dispute_resolvers.contains(signer) {
        return Err(ContractError::DisputeResolverCannotDisputeTheEscrow);
    }

    let is_authorized = escrow.roles.approvers.contains(signer)
        || escrow.roles.service_providers.contains(signer)
        || signer == &escrow.roles.platform
        || escrow.roles.release_signers.contains(signer)
        || signer == &escrow.roles.receiver;

    if !is_authorized {
        return Err(ContractError::UnauthorizedToChangeDisputeFlag);
    }

    Ok(())
}
