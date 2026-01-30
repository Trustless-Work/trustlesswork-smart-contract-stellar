use crate::error::ContractError;
use crate::storage::types::DataKey;
use crate::{core::escrow::EscrowManager, storage::types::{Escrow, MilestoneUpdate}};
use soroban_sdk::{Address, Env, Vec};

use super::validators::milestone::{
    validate_milestone_flag_change_conditions, validate_milestone_status_change_conditions,
    validate_and_convert_milestone_index,
};

pub struct MilestoneManager;

impl MilestoneManager {
    pub fn change_milestone_status(
        e: &Env,
        milestone_updates: Vec<MilestoneUpdate>,
        service_provider: Address,
    ) -> Result<Escrow, ContractError> {
        service_provider.require_auth();
        let mut existing_escrow = EscrowManager::get_escrow(e)?;

        validate_milestone_status_change_conditions(
            &existing_escrow,
            &milestone_updates,
            &service_provider,
        )?;

        for i in 0..milestone_updates.len() {
            let update = milestone_updates.get(i).unwrap();
            let idx = validate_and_convert_milestone_index(
                update.index,
                existing_escrow.milestones.len(),
            )?;

            let mut milestone_to_update = existing_escrow
                .milestones
                .get(idx)
                .ok_or(ContractError::MilestoneToUpdateDoesNotExist)?;

            if let Some(ref evidence) = update.evidence {
                milestone_to_update.evidence = evidence.clone();
            }

            milestone_to_update.status = update.status.clone();

            existing_escrow
                .milestones
                .set(idx, milestone_to_update);
        }

        e.storage()
            .instance()
            .set(&DataKey::Escrow, &existing_escrow);

        Ok(existing_escrow)
    }

    pub fn change_milestone_approved_flag(
        e: &Env,
        milestone_indexes: Vec<i128>,
        approver: Address,
    ) -> Result<Escrow, ContractError> {
        approver.require_auth();
        let mut existing_escrow = EscrowManager::get_escrow(e)?;

        validate_milestone_flag_change_conditions(
            &existing_escrow,
            &milestone_indexes,
            &approver,
        )?;

        for i in 0..milestone_indexes.len() {
            let milestone_index = milestone_indexes.get(i).unwrap();
            let idx = validate_and_convert_milestone_index(
                milestone_index,
                existing_escrow.milestones.len(),
            )?;

            let mut milestone_to_update = existing_escrow
                .milestones
                .get(idx)
                .unwrap();
            
            milestone_to_update.approved = true;
            existing_escrow
                .milestones
                .set(idx, milestone_to_update);
        }
        
        e.storage()
            .instance()
            .set(&DataKey::Escrow, &existing_escrow);

        Ok(existing_escrow)
    }
}
