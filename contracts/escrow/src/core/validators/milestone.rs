use soroban_sdk::{Address, Vec};

use crate::{
    error::ContractError,
    storage::types::{Escrow, MilestoneUpdate},
};

#[inline]
pub fn validate_and_convert_milestone_index(
    milestone_index: i128,
    milestones_len: u32,
) -> Result<u32, ContractError> {
    if milestone_index < 0 {
        return Err(ContractError::InvalidMileStoneIndex);
    }

    let idx = u32::try_from(milestone_index)
        .map_err(|_| ContractError::InvalidMileStoneIndex)?;

    if idx >= milestones_len {
        return Err(ContractError::InvalidMileStoneIndex);
    }

    Ok(idx)
}

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
        return Err(ContractError::NoMilestoneDefined);
    }

    for i in 0..milestone_updates.len() {
        let update = milestone_updates.get(i).unwrap();
        
        if update.status.is_empty() {
            return Err(ContractError::EmptyMilestoneStatus);
        }
        
        let idx = validate_and_convert_milestone_index(
            update.index,
            escrow.milestones.len(),
        )?;
        
        let _milestone = escrow
            .milestones
            .get(idx)
            .ok_or(ContractError::MilestoneToUpdateDoesNotExist)?;
    }

    Ok(())
}

#[inline]
pub fn validate_milestone_flag_change_conditions(
    escrow: &Escrow,
    milestone_indexes: &soroban_sdk::Vec<i128>,
    approver: &Address,
) -> Result<(), ContractError> {
    if approver != &escrow.roles.approver {
        return Err(ContractError::OnlyApproverChangeMilstoneFlag);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMilestoneDefined);
    }

    for i in 0..milestone_indexes.len() {
        let milestone_index = milestone_indexes.get(i).unwrap();
        
        let idx = validate_and_convert_milestone_index(
            milestone_index,
            escrow.milestones.len(),
        )?;
        
        let milestone = escrow
            .milestones
            .get(idx)
            .ok_or(ContractError::MilestoneToApproveDoesNotExist)?;
        
        if milestone.approved {
            return Err(ContractError::MilestoneHasAlreadyBeenApproved);
        }

        if milestone.status.is_empty() {
            return Err(ContractError::EmptyMilestoneStatus);
        }
    }

    Ok(())
}
