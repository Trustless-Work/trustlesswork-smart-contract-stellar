use soroban_sdk::Address;

use crate::{
    error::ContractError,
    storage::types::{Escrow, Milestone},
};

#[inline]
pub fn validate_milestone_status_change_conditions(
    escrow: &Escrow,
    service_provider: &Address,
    milestone_index: &u32
) -> Result<(), ContractError> {
    if service_provider != &escrow.roles.service_provider {
        return Err(ContractError::OnlyServiceProviderChangeMilstoneStatus);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMilestoneDefined);
    }

    let update = escrow.milestones.get(*milestone_index).unwrap();
        
    if update.status.is_empty() {
        return Err(ContractError::EmptyMilestoneStatus);
    }

    if *milestone_index < 0 {
        return Err(ContractError::InvalidMileStoneIndex);
    }

    Ok(())
}

#[inline]
pub fn validate_milestone_flag_change_conditions(
    escrow: &Escrow,
    milestone: &Milestone,
    approver: &Address,
    milestone_index: &u32
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
        
    if *milestone_index < 0 {
        return Err(ContractError::InvalidMileStoneIndex);
    }
        
    // Check that index doesn't overflow when casting to u32 and is within bounds
    if *milestone_index >= escrow.milestones.len() {
        return Err(ContractError::MilestoneToApproveDoesNotExist);
    }

    Ok(())
}
