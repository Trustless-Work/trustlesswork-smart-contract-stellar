use soroban_sdk::{Address, Vec};

use crate::{
    error::MilestoneError,
    storage::types::{Escrow, MilestoneStatusUpdate},
};

#[inline]
pub fn validate_batch_milestone_status_change(
    escrow: &Escrow,
    service_provider: &Address,
    updates: &Vec<MilestoneStatusUpdate>,
) -> Result<(), MilestoneError> {
    if updates.is_empty() {
        return Err(MilestoneError::BatchMilestoneUpdateEmpty);
    }

    if !escrow.roles.service_providers.contains(service_provider) {
        return Err(MilestoneError::OnlyServiceProviderCanChangeMilestoneStatus);
    }

    if escrow.milestones.is_empty() {
        return Err(MilestoneError::NoMilestoneDefined);
    }

    for update in updates.iter() {
        if update.new_status.is_empty() {
            return Err(MilestoneError::EmptyMilestoneStatus);
        }

        if update.milestone_index >= escrow.milestones.len() {
            return Err(MilestoneError::MilestoneToUpdateDoesNotExist);
        }
    }

    Ok(())
}

#[inline]
pub fn validate_batch_milestone_approve(
    escrow: &Escrow,
    approver: &Address,
    milestone_indices: &Vec<u32>,
) -> Result<(), MilestoneError> {
    if milestone_indices.is_empty() {
        return Err(MilestoneError::BatchMilestoneApproveEmpty);
    }

    if !escrow.roles.approvers.contains(approver) {
        return Err(MilestoneError::UnauthorizedApprover);
    }

    if escrow.milestones.is_empty() {
        return Err(MilestoneError::NoMilestoneDefined);
    }

    for index in milestone_indices.iter() {
        if index >= escrow.milestones.len() {
            return Err(MilestoneError::MilestoneToApproveDoesNotExist);
        }

        let milestone = escrow.milestones.get(index).unwrap();

        if milestone.approvals.target > 0
            && milestone.approvals.approval_count >= milestone.approvals.target
        {
            return Err(MilestoneError::MilestoneHasAlreadyBeenApproved);
        }

        if milestone.approvals.approvers.contains(approver) {
            return Err(MilestoneError::ApproverAlreadyApprovedMilestone);
        }
    }

    Ok(())
}
