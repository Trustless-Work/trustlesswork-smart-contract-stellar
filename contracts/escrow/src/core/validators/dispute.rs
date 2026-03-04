use soroban_sdk::{Address, Map};

use crate::{
    error::ContractError,
    storage::types::{Escrow, Roles},
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

    if dispute_resolver != &escrow.roles.dispute_resolver {
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

    if dispute_resolver != &escrow.roles.dispute_resolver {
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

    let Roles {
        approver,
        service_provider,
        platform,
        release_signer,
        dispute_resolver,
        receiver,
    } = &escrow.roles;

    let is_authorized = signer == approver
        || signer == service_provider
        || signer == platform
        || signer == release_signer
        || signer == dispute_resolver
        || signer == receiver;

    if !is_authorized {
        return Err(ContractError::UnauthorizedToChangeDisputeFlag);
    }

    if signer == dispute_resolver {
        return Err(ContractError::DisputeResolverCannotDisputeTheEscrow);
    }

    Ok(())
}
