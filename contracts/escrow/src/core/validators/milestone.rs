use soroban_sdk::{Address, Vec};

use crate::{
    error::MilestoneError,
    storage::types::{Escrow, MilestoneStatusUpdate},
};

const MAX_STATUS_LEN: u32 = 50;
const MAX_EVIDENCE_LEN: u32 = 500;
const MAX_BATCH_SIZE: u32 = 50;

#[inline]
pub fn validate_batch_milestone_status_change(
    escrow: &Escrow,
    service_provider: &Address,
    updates: &Vec<MilestoneStatusUpdate>,
) -> Result<(), MilestoneError> {
    if updates.is_empty() {
        return Err(MilestoneError::BatchMilestoneUpdateEmpty);
    }

    if updates.len() > MAX_BATCH_SIZE {
        return Err(MilestoneError::BatchTooLarge);
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

        if update.new_status.len() > MAX_STATUS_LEN {
            return Err(MilestoneError::StringTooLong);
        }

        if let Some(ref evidence) = update.new_evidence {
            if evidence.len() > MAX_EVIDENCE_LEN {
                return Err(MilestoneError::StringTooLong);
            }
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

    if milestone_indices.len() > MAX_BATCH_SIZE {
        return Err(MilestoneError::BatchTooLarge);
    }

    for i in 0..milestone_indices.len() {
        for j in (i + 1)..milestone_indices.len() {
            if milestone_indices.get(i).unwrap() == milestone_indices.get(j).unwrap() {
                return Err(MilestoneError::DuplicateMilestoneIndex);
            }
        }
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

        if milestone.approvals.approved_by.contains(approver) {
            return Err(MilestoneError::ApproverAlreadyApprovedMilestone);
        }
    }

    Ok(())
}
