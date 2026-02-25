use soroban_sdk::{Address, Vec};

use crate::{
    error::ContractError,
    storage::types::{Escrow, Milestone, MilestoneUpdate},
};

#[inline]
pub fn validate_milestone_status_change_conditions(
    escrow: &Escrow,
    milestone_updates: &Vec<MilestoneUpdate>,
    service_provider: &Address,
) -> Result<(), ContractError> {
    if service_provider != &escrow.roles.service_provider {
        return Err(ContractError::OnlyServiceProviderChangeMilstoneStatus);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMileStoneDefined);
    }

    for i in 0..milestone_updates.len() {
        let update = milestone_updates
            .get(i)
            .ok_or(ContractError::MilestoneToUpdateDoesNotExist)?;

        if update.status.is_empty() {
            return Err(ContractError::EmptyMilestoneStatus);
        }

        let idx = validate_and_convert_milestone_index(update.index, escrow.milestones.len())?;

        let _milestone = escrow
            .milestones
            .get(idx)
            .ok_or(ContractError::MilestoneToUpdateDoesNotExist)?;
    }

    Ok(())
}

#[inline]
pub fn validate_and_convert_milestone_index(
    milestone_index: u32,
    milestones_len: u32,
) -> Result<u32, ContractError> {
    let idx = u32::try_from(milestone_index).map_err(|_| ContractError::InvalidMileStoneIndex)?;

    if idx >= milestones_len {
        return Err(ContractError::InvalidMileStoneIndex);
    }

    Ok(idx)
}

#[inline]
pub fn validate_milestone_flag_change_conditions(
    escrow: &Escrow,
    milestone: &Milestone,
    approver: &Address,
) -> Result<(), ContractError> {
    if approver != &escrow.roles.approver {
        return Err(ContractError::OnlyApproverChangeMilstoneFlag);
    }

    if milestone.flags.approved {
        return Err(ContractError::MilestoneHasAlreadyBeenApproved);
    }

    if milestone.status.is_empty() {
        return Err(ContractError::EmptyMilestoneStatus);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMileStoneDefined);
    }

    Ok(())
}
