use crate::error::ContractError;
use crate::storage::types::{DataKey, Escrow};
use crate::core::escrow::EscrowManager;
use soroban_sdk::{Address, Env, String, Vec};

use super::validators::milestone::{
    validate_milestone_flag_change_conditions, validate_milestone_status_change_conditions,
};

pub struct MilestoneManager;

impl MilestoneManager {
    pub fn change_milestone_status(
        e: &Env,
        milestone_index: i128,
        new_status: String,
        new_evidence: Option<String>,
        service_provider: Address,
    ) -> Result<Escrow, ContractError> {
        service_provider.require_auth();
        let mut existing_escrow = EscrowManager::get_escrow(e)?;

        validate_milestone_status_change_conditions(&existing_escrow, &service_provider)?;

        let mut milestone_to_update = existing_escrow
            .milestones
            .get(milestone_index as u32)
            .ok_or(ContractError::InvalidMileStoneIndex)?;

        if let Some(evidence) = new_evidence {
            milestone_to_update.evidence = evidence;
        }

        milestone_to_update.status = new_status;

        existing_escrow
            .milestones
            .set(milestone_index as u32, milestone_to_update);
        e.storage().instance().set(&DataKey::Escrow, &existing_escrow);

        Ok(existing_escrow)
    }

    pub fn change_milestone_approved_flags(
        e: &Env,
        milestone_indexes: Vec<i128>,
        approver: Address,
    ) -> Result<Escrow, ContractError> {
        approver.require_auth();
        let mut existing_escrow = EscrowManager::get_escrow(e)?;

        validate_milestone_flag_change_conditions(
            &existing_escrow,
            &milestone_indexes, 
            &approver
        )?;

        for i in 0..milestone_indexes.len() {
            let milestone_index = milestone_indexes.get(i).unwrap();
            let mut milestone_to_update = existing_escrow
                .milestones
                .get(milestone_index as u32)
                .unwrap();
            
            milestone_to_update.flags.approved = true;
            existing_escrow
                .milestones
                .set(milestone_index as u32, milestone_to_update);
        }

        e.storage().instance().set(&DataKey::Escrow, &existing_escrow);

        Ok(existing_escrow)
    }
}
