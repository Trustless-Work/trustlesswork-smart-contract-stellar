use crate::error::ContractError;
use crate::storage::types::{DataKey, MilestoneStatusUpdate};
use crate::{core::escrow::EscrowManager, storage::types::Escrow};
use soroban_sdk::{Address, Env, Vec};

use super::validators::milestone::{
    validate_batch_milestone_approve, validate_batch_milestone_status_change,
};

pub struct MilestoneManager;

impl MilestoneManager {
    pub fn change_milestone_status(
        e: &Env,
        updates: Vec<MilestoneStatusUpdate>,
        service_provider: Address,
    ) -> Result<Escrow, ContractError> {
        let mut existing_escrow = EscrowManager::get_escrow(e)?;

        validate_batch_milestone_status_change(&existing_escrow, &service_provider, &updates)?;

        service_provider.require_auth();

        for update in updates.iter() {
            let mut milestone_to_update = existing_escrow
                .milestones
                .get(update.milestone_index)
                .ok_or(ContractError::InvalidMileStoneIndex)?;

            if let Some(evidence) = update.new_evidence {
                milestone_to_update.evidence = evidence;
            }

            milestone_to_update.status = update.new_status;

            existing_escrow
                .milestones
                .set(update.milestone_index, milestone_to_update);
        }

        e.storage()
            .persistent()
            .set(&DataKey::Escrow, &existing_escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        Ok(existing_escrow)
    }

    pub fn approve_milestones(
        e: &Env,
        milestone_indices: Vec<u32>,
        approver: Address,
    ) -> Result<Escrow, ContractError> {
        let mut existing_escrow = EscrowManager::get_escrow(e)?;

        validate_batch_milestone_approve(&existing_escrow, &approver, &milestone_indices)?;

        approver.require_auth();

        for index in milestone_indices.iter() {
            let mut milestone = existing_escrow
                .milestones
                .get(index)
                .ok_or(ContractError::InvalidMileStoneIndex)?;

            milestone.approvals.approvers.push_back(approver.clone());
            milestone.approvals.approval_count += 1;

            existing_escrow.milestones.set(index, milestone);
        }

        e.storage()
            .persistent()
            .set(&DataKey::Escrow, &existing_escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        Ok(existing_escrow)
    }
}
