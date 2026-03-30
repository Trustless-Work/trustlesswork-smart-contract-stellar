use soroban_sdk::{Address, Vec};

use crate::{
    error::ContractError,
    storage::types::{Escrow, MilestoneStatusUpdate},
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

    if !escrow.roles.service_providers.contains(service_provider) {
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
pub fn validate_batch_milestone_approve(
    escrow: &Escrow,
    approver: &Address,
    milestone_indices: &Vec<u32>,
) -> Result<(), ContractError> {
    if milestone_indices.is_empty() {
        return Err(ContractError::BatchMilestoneApproveEmpty);
    }

    if !escrow.roles.approvers.contains(approver) {
        return Err(ContractError::UnauthorizedApprover);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMilestoneDefined);
    }

    for index in milestone_indices.iter() {
        if index >= escrow.milestones.len() {
            return Err(ContractError::MilestoneToApproveDoesNotExist);
        }

        let milestone = escrow.milestones.get(index).unwrap();

        if milestone.approvals.quorum > 0
            && milestone.approvals.approval_count >= milestone.approvals.quorum
        {
            return Err(ContractError::MilestoneHasAlreadyBeenApproved);
        }

        if milestone.approvals.approvers.contains(approver) {
            return Err(ContractError::ApproverAlreadyApprovedMilestone);
        }
    }

    Ok(())
}
