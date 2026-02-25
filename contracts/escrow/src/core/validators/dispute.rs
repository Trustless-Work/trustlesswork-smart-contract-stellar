use soroban_sdk::{Address, Map};

use crate::{
    core::validators::milestone::validate_and_convert_milestone_index,
    error::ContractError,
    storage::types::{Escrow, Milestone, Roles},
};

const MAX_DISTRIBUTIONS: u32 = 50;

#[inline]
pub fn validate_dispute_resolution_conditions(
    escrow: &Escrow,
    milestone: &Milestone,
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

    if milestone.flags.released {
        return Err(ContractError::MilestoneAlreadyReleased);
    }

    if milestone.flags.resolved {
        return Err(ContractError::MilestoneAlreadyResolved);
    }

    if !milestone.flags.disputed {
        return Err(ContractError::MilestoneNotInDispute);
    }

    if total > milestone.amount {
        return Err(ContractError::TotalDisputeFundsMustNotExceedTheMilestoneAmount);
    }

    if current_balance < total {
        return Err(ContractError::InsufficientFundsForResolution);
    }

    if total <= 0 {
        return Err(ContractError::TotalAmountCannotBeZero);
    }

    Ok(())
}

#[inline]
pub fn validate_withdraw_remaining_funds_conditions(
    escrow: &Escrow,
    dispute_resolver: &Address,
    all_processed: bool,
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
pub fn validate_dispute_flag_change_conditions(
    escrow: &Escrow,
    milestone_index: u32,
    signer: &Address,
) -> Result<(), ContractError> {
    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMileStoneDefined);
    }

    let idx = validate_and_convert_milestone_index(milestone_index, escrow.milestones.len())?;

    let milestone = escrow
        .milestones
        .get(idx)
        .ok_or(ContractError::InvalidMileStoneIndex)?;

    if milestone.flags.released {
        return Err(ContractError::MilestoneAlreadyReleased);
    }
    if milestone.flags.resolved {
        return Err(ContractError::MilestoneAlreadyResolved);
    }
    if milestone.flags.disputed {
        return Err(ContractError::MilestoneAlreadyInDispute);
    }

    let Roles {
        approver,
        service_provider,
        platform_address,
        release_signer,
        dispute_resolver,
    } = &escrow.roles;

    let is_authorized = signer == approver
        || signer == service_provider
        || signer == platform_address
        || signer == release_signer
        || signer == dispute_resolver
        || signer == &milestone.receiver;

    if !is_authorized {
        return Err(ContractError::UnauthorizedToChangeDisputeFlag);
    }

    if signer == dispute_resolver {
        return Err(ContractError::DisputeResolverCannotDisputeTheMilestone);
    }

    Ok(())
}
