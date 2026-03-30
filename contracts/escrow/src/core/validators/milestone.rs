use soroban_sdk::{Address, Vec};

use crate::{
    error::ContractError,
    storage::types::{Escrow, Milestone, MilestoneStatusUpdate},
};

#[inline]
pub fn validate_batch_milestone_status_change(
    escrow: &Escrow,
    service_provider: &Address,
    updates: &Vec<MilestoneStatusUpdate>,
) -> Result<(), ContractError> {
    if updates.is_empty() {
        return Err(ContractError::BatchMilestoneUpdateEmpty);
    }

    if service_provider != &escrow.roles.service_provider {
        return Err(ContractError::OnlyServiceProviderChangeMilstoneStatus);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMilestoneDefined);
    }

    for update in updates.iter() {
        if update.new_status.is_empty() {
            return Err(ContractError::EmptyMilestoneStatus);
        }

        if update.milestone_index >= escrow.milestones.len() {
            return Err(ContractError::MilestoneToUpdateDoesNotExist);
        }
    }

    Ok(())
}

#[inline]
pub fn validate_milestone_flag_change_conditions(
    escrow: &Escrow,
    milestone: &Milestone,
    approver: &Address,
    milestone_index: &u32,
) -> Result<(), ContractError> {
    if approver != &escrow.roles.approver {
        return Err(ContractError::OnlyApproverChangeMilstoneFlag);
    }

    if milestone.approved {
        return Err(ContractError::MilestoneHasAlreadyBeenApproved);
    }

    if milestone.status.is_empty() {
        return Err(ContractError::EmptyMilestoneStatus);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMilestoneDefined);
    }

    if *milestone_index >= escrow.milestones.len() {
        return Err(ContractError::MilestoneToApproveDoesNotExist);
    }

    Ok(())
}
