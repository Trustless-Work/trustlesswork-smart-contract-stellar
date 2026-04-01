use soroban_sdk::{Address, Map, Vec};

use crate::{
    error::EscrowError,
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
) -> Result<(), EscrowError> {
    if distributions.len() > MAX_DISTRIBUTIONS {
        return Err(EscrowError::TooManyDistributions);
    }

    if !escrow.roles.dispute_resolvers.contains(dispute_resolver) {
        return Err(EscrowError::OnlyDisputeResolverCanExecuteThisFunction);
    }

    if !all_processed {
        return Err(EscrowError::EscrowNotFullyProcessed);
    }

    if total <= 0 {
        return Err(EscrowError::TotalAmountCannotBeZero);
    }

    if current_balance < total {
        return Err(EscrowError::InsufficientFundsForResolution);
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
) -> Result<(), EscrowError> {
    if distributions.len() > MAX_DISTRIBUTIONS {
        return Err(EscrowError::TooManyDistributions);
    }

    if !escrow.roles.dispute_resolvers.contains(dispute_resolver) {
        return Err(EscrowError::OnlyDisputeResolverCanExecuteThisFunction);
    }

    if !escrow.milestones.iter().any(|m| m.dispute.is_disputed) {
        return Err(EscrowError::EscrowNotInDispute);
    }

    if current_balance < total {
        return Err(EscrowError::InsufficientFundsForResolution);
    }

    if total != current_balance {
        return Err(EscrowError::DistributionsMustEqualEscrowBalance);
    }

    if total <= 0 {
        return Err(EscrowError::TotalAmountCannotBeZero);
    }

    Ok(())
}

#[inline]
pub fn validate_batch_milestone_dispute_conditions(
    escrow: &Escrow,
    signer: &Address,
    milestone_indices: &Vec<u32>,
) -> Result<(), EscrowError> {
    if milestone_indices.is_empty() {
        return Err(EscrowError::BatchMilestoneDisputeEmpty);
    }

    if escrow.milestones.iter().any(|m| m.dispute.resolved) {
        return Err(EscrowError::EscrowAlreadyResolved);
    }

    if escrow.roles.dispute_resolvers.contains(signer) {
        return Err(EscrowError::DisputeResolverCannotDisputeTheEscrow);
    }

    let is_authorized = escrow.roles.approvers.contains(signer)
        || escrow.roles.service_providers.contains(signer)
        || signer == &escrow.roles.platform
        || escrow.roles.release_signers.contains(signer)
        || signer == &escrow.roles.receiver;

    if !is_authorized {
        return Err(EscrowError::UnauthorizedToChangeDisputeFlag);
    }

    for index in milestone_indices.iter() {
        if index >= escrow.milestones.len() {
            return Err(EscrowError::InvalidMilestoneIndex);
        }

        let milestone = escrow.milestones.get(index).unwrap();

        if milestone.dispute.is_disputed {
            return Err(EscrowError::MilestoneAlreadyDisputed);
        }

        if milestone.released {
            return Err(EscrowError::MilestoneAlreadyReleased);
        }
    }

    Ok(())
}

#[inline]
pub fn validate_dispute_flag_change_conditions(
    escrow: &Escrow,
    signer: &Address,
) -> Result<(), EscrowError> {
    if escrow.milestones.iter().all(|m| m.dispute.is_disputed) {
        return Err(EscrowError::EscrowAlreadyInDispute);
    }

    if escrow.milestones.iter().any(|m| m.dispute.resolved) {
        return Err(EscrowError::EscrowAlreadyResolved);
    }

    if escrow.roles.dispute_resolvers.contains(signer) {
        return Err(EscrowError::DisputeResolverCannotDisputeTheEscrow);
    }

    let is_authorized = escrow.roles.approvers.contains(signer)
        || escrow.roles.service_providers.contains(signer)
        || signer == &escrow.roles.platform
        || escrow.roles.release_signers.contains(signer)
        || signer == &escrow.roles.receiver;

    if !is_authorized {
        return Err(EscrowError::UnauthorizedToChangeDisputeFlag);
    }

    Ok(())
}
