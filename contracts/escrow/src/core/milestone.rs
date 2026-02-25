use crate::error::ContractError;
use crate::storage::types::DataKey;
use crate::{core::escrow::EscrowManager, storage::types::Escrow};
use soroban_sdk::{Address, Env, String};

use super::validators::milestone::{
    validate_milestone_flag_change_conditions, validate_milestone_status_change_conditions,
};

pub struct MilestoneManager;

impl MilestoneManager {
    pub fn change_milestone_status(
        e: &Env,
        milestone_index: u32,
        new_status: String,
        new_evidence: Option<String>,
        service_provider: Address,
    ) -> Result<Escrow, ContractError> {
        let mut existing_escrow = EscrowManager::get_escrow(e)?;

        validate_milestone_status_change_conditions(
            &existing_escrow,
            &service_provider,
            &milestone_index,
            &new_status,
        )?;

        service_provider.require_auth();

        let mut milestone_to_update = existing_escrow
            .milestones
            .get(milestone_index)
            .ok_or(ContractError::InvalidMileStoneIndex)?;

        if let Some(evidence) = new_evidence {
            milestone_to_update.evidence = evidence;
        }

        milestone_to_update.status = new_status;

        existing_escrow
            .milestones
            .set(milestone_index, milestone_to_update);
        e.storage()
            .persistent()
            .set(&DataKey::Escrow, &existing_escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        Ok(existing_escrow)
    }

    pub fn change_milestone_approved_flag(
        e: &Env,
        milestone_index: u32,
        approver: Address,
    ) -> Result<Escrow, ContractError> {
        let mut existing_escrow = EscrowManager::get_escrow(e)?;

        let mut milestone_to_update = existing_escrow
            .milestones
            .get(milestone_index)
            .ok_or(ContractError::InvalidMileStoneIndex)?;

        validate_milestone_flag_change_conditions(
            &existing_escrow,
            &milestone_to_update,
            &approver,
            &milestone_index,
        )?;

        approver.require_auth();

        milestone_to_update.approved = true;

        existing_escrow
            .milestones
            .set(milestone_index, milestone_to_update);
        e.storage()
            .persistent()
            .set(&DataKey::Escrow, &existing_escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        Ok(existing_escrow)
    }
}
