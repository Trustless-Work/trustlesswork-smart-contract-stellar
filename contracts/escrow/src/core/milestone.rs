use crate::core::escrow::EscrowManager;
use crate::error::ContractError;
use crate::storage::types::{DataKey, Escrow, MilestoneUpdate};
use soroban_sdk::{Address, Env, Vec};

use super::validators::milestone::{
    validate_and_convert_milestone_index, validate_milestone_flag_change_conditions,
    validate_milestone_status_change_conditions,
};

pub struct MilestoneManager;

impl MilestoneManager {
    pub fn change_milestone_status(
        e: &Env,
        milestone_updates: Vec<MilestoneUpdate>,
        service_provider: Address,
    ) -> Result<Escrow, ContractError> {
        let mut existing_escrow = EscrowManager::get_escrow(e)?;

        validate_milestone_status_change_conditions(
            &existing_escrow,
            &milestone_updates,
            &service_provider,
        )?;

        service_provider.require_auth();

        for i in 0..milestone_updates.len() {
            let update = milestone_updates.get(i).unwrap();
            let idx = validate_and_convert_milestone_index(
                update.index,
                existing_escrow.milestones.len(),
            )?;

            let mut milestone_to_update = existing_escrow.milestones.get(idx).unwrap();

            if let Some(ref evidence) = update.evidence {
                milestone_to_update.evidence = evidence.clone();
            }

            milestone_to_update.status = update.status.clone();

            existing_escrow.milestones.set(idx, milestone_to_update);
        }

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
        )?;

        approver.require_auth();

        milestone_to_update.flags.approved = true;

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
