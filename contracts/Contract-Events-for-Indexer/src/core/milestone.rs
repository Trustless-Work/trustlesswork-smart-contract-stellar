use crate::core::escrow::EscrowManager;
use crate::error::ContractError;
use crate::events::escrows_by_contract_id;
use crate::storage::types::DataKey;
use soroban_sdk::{Address, Env, String};

use super::validators::milestone::{
    validate_milestone_flag_change_conditions, validate_milestone_status_change_conditions,
};

pub struct MilestoneManager;

impl MilestoneManager {
    pub fn change_milestone_status(
        e: Env,
        milestone_index: i128,
        new_status: String,
        new_evidence: Option<String>,
        service_provider: Address,
    ) -> Result<(), ContractError> {
        service_provider.require_auth();
        let mut existing_escrow = EscrowManager::get_escrow(e.clone())?;

        validate_milestone_status_change_conditions(
            &existing_escrow,
            milestone_index,
            &service_provider,
        )?;

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
        e.storage()
            .instance()
            .set(&DataKey::Escrow, &existing_escrow);
        escrows_by_contract_id(&e, existing_escrow.engagement_id.clone(), existing_escrow);
        Ok(())
    }

    pub fn change_milestone_approved_flag(
        e: Env,
        milestone_index: i128,
        new_flag: bool,
        approver: Address,
    ) -> Result<(), ContractError> {
        approver.require_auth();
        let mut existing_escrow = EscrowManager::get_escrow(e.clone())?;

        validate_milestone_flag_change_conditions(&existing_escrow, milestone_index, &approver)?;

        let mut milestone_to_update = existing_escrow
            .milestones
            .get(milestone_index as u32)
            .ok_or(ContractError::InvalidMileStoneIndex)?;

        milestone_to_update.approved = new_flag;

        existing_escrow
            .milestones
            .set(milestone_index as u32, milestone_to_update);
        e.storage()
            .instance()
            .set(&DataKey::Escrow, &existing_escrow);
        escrows_by_contract_id(&e, existing_escrow.engagement_id.clone(), existing_escrow);
        Ok(())
    }
}
