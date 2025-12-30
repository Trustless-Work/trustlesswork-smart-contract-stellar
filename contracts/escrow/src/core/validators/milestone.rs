use soroban_sdk::{Address, Vec};

use crate::{
    error::ContractError,
    storage::types::{Escrow, MilestoneUpdate},
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
        let update = milestone_updates.get(i).unwrap();
        
        if update.status.is_empty() {
            return Err(ContractError::EmptyMilestoneStatus);
        }
        
        if update.index < 0 {
            return Err(ContractError::InvalidMileStoneIndex);
        }
        
        let _milestone = escrow
            .milestones
            .get(update.index as u32)
            .ok_or(ContractError::MilestoneToUpdateDoesNotExist)?;
    }

    Ok(())
}

#[inline]
pub fn validate_milestone_flag_change_conditions(
    escrow: &Escrow,
    milestone_indexes: &Vec<i128>,
    approver: &Address,
) -> Result<(), ContractError> {
    if approver != &escrow.roles.approver {
        return Err(ContractError::OnlyApproverChangeMilstoneFlag);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMileStoneDefined);
    }

    for i in 0..milestone_indexes.len() {
        let milestone_index = milestone_indexes.get(i).unwrap();

        if milestone_index < 0 {
            return Err(ContractError::InvalidMileStoneIndex);
        }

        let milestone = escrow
            .milestones
            .get(milestone_index as u32)
            .ok_or(ContractError::MilestoneToApproveDoesNotExist)?;

        if milestone.flags.approved {
            return Err(ContractError::MilestoneHasAlreadyBeenApproved);
        }

        if milestone.status.is_empty() {
            return Err(ContractError::EmptyMilestoneStatus);
        }
    }

    Ok(())
}
