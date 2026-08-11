use soroban_sdk::{Address, Map, Vec};

use crate::{
    error::EscrowError,
    modules::math::{BasicArithmetic, BasicMath},
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
    distributions: &Map<Address, i128>,
) -> Result<(), EscrowError> {
    if distributions.len() > MAX_DISTRIBUTIONS {
        return Err(EscrowError::TooManyDistributions);
    }

    if !escrow.roles.dispute_resolvers.contains(dispute_resolver) {
        return Err(EscrowError::OnlyDisputeResolverCanExecuteThisFunction);
    }

    if !escrow
        .milestones
        .iter()
        .any(|m| m.dispute.is_disputed || m.dispute.resolved)
    {
        return Err(EscrowError::EscrowNotInDispute);
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
    milestone_indices: &Vec<u32>,
) -> Result<(), EscrowError> {
    if distributions.len() > MAX_DISTRIBUTIONS {
        return Err(EscrowError::TooManyDistributions);
    }

    if !escrow.roles.dispute_resolvers.contains(dispute_resolver) {
        return Err(EscrowError::OnlyDisputeResolverCanExecuteThisFunction);
    }

    if milestone_indices.is_empty() {
        return Err(EscrowError::EscrowNotInDispute);
    }

    for i in 0..milestone_indices.len() {
        for j in (i + 1)..milestone_indices.len() {
            if milestone_indices.get(i).unwrap() == milestone_indices.get(j).unwrap() {
                return Err(EscrowError::InvalidMilestoneIndex);
            }
        }
    }

    let mut milestone_amount_total: i128 = 0;
    for index in milestone_indices.iter() {
        if index >= escrow.milestones.len() {
            return Err(EscrowError::InvalidMilestoneIndex);
        }
        let milestone = escrow.milestones.get(index).unwrap();
        if !milestone.dispute.is_disputed {
            return Err(EscrowError::EscrowNotInDispute);
        }
        if milestone.dispute.resolved {
            return Err(EscrowError::EscrowAlreadyResolved);
        }
        milestone_amount_total = BasicMath::safe_add(milestone_amount_total, milestone.amount)?;
    }

    if total > milestone_amount_total {
        return Err(EscrowError::DistributionsMustEqualEscrowBalance);
    }

    if current_balance < total {
        return Err(EscrowError::InsufficientFundsForResolution);
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

    if milestone_indices.len() > escrow.milestones.len() {
        return Err(EscrowError::TooManyMilestones);
    }

    for i in 0..milestone_indices.len() {
        for j in (i + 1)..milestone_indices.len() {
            if milestone_indices.get(i).unwrap() == milestone_indices.get(j).unwrap() {
                return Err(EscrowError::InvalidMilestoneIndex);
            }
        }
    }

    if escrow.roles.dispute_resolvers.contains(signer) {
        return Err(EscrowError::DisputeResolverCannotDisputeTheEscrow);
    }

    let is_global_role = escrow.roles.approvers.contains(signer)
        || escrow.roles.service_providers.contains(signer)
        || signer == &escrow.roles.platform
        || escrow.roles.release_signers.contains(signer);

    if !is_global_role {
        let is_receiver_for_all = milestone_indices.iter().all(|index| {
            escrow
                .milestones
                .get(index)
                .is_some_and(|m| m.receiver.stellar_address.as_ref() == Some(signer))
        });

        if !is_receiver_for_all {
            return Err(EscrowError::UnauthorizedToChangeDisputeFlag);
        }
    }

    for index in milestone_indices.iter() {
        if index >= escrow.milestones.len() {
            return Err(EscrowError::InvalidMilestoneIndex);
        }

        let milestone = escrow.milestones.get(index).unwrap();

        if milestone.dispute.resolved {
            return Err(EscrowError::EscrowAlreadyResolved);
        }

        if milestone.dispute.is_disputed {
            return Err(EscrowError::MilestoneAlreadyDisputed);
        }

        if milestone.released {
            return Err(EscrowError::MilestoneAlreadyReleased);
        }
    }

    Ok(())
}
