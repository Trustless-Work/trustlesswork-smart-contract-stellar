use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Map, String, Vec};

use crate::core::escrow::EscrowManager;
use crate::core::validators::dispute::{validate_withdraw_remaining_funds_conditions};
use crate::error::EscrowError;
use crate::modules::fee::distribution::calculate_and_distribute_fees;
use crate::modules::{
    fee::{FeeCalculator, FeeCalculatorTrait, StandardFeeResult},
    math::{BasicArithmetic, BasicMath},
};
use crate::storage::types::{DataKey, Escrow};

use super::validators::dispute::{
    validate_batch_milestone_dispute_conditions,
    validate_dispute_resolution_conditions,
};

pub struct DisputeManager;

impl DisputeManager {
    pub fn withdraw_remaining_funds(
        e: &Env,
        dispute_resolver: Address,
        trustless_work_address: Address,
        distributions: Map<Address, i128>,
    ) -> Result<(Escrow, StandardFeeResult, Vec<(Address, i128)>), EscrowError> {
        if e.storage().persistent().has(&DataKey::Reentrancy) {
            return Err(EscrowError::FlagsMustBeFalse);
        }
        e.storage().persistent().set(&DataKey::Reentrancy, &true);

        let escrow = EscrowManager::get_escrow(e)?;
        let contract_address = e.current_contract_address();

        let all_processed = escrow
            .milestones
            .iter()
            .all(|m| m.released || m.dispute.resolved);

        let token_client = TokenClient::new(&e, &escrow.trustline.address);
        let current_balance = token_client.balance(&contract_address);
        let mut total: i128 = 0;
        for (_addr, amount) in distributions.iter() {
            if amount <= 0 {
                return Err(EscrowError::AmountsToBeTransferredShouldBePositive);
            }
            total = BasicMath::safe_add(total, amount)?;
        }

        validate_withdraw_remaining_funds_conditions(
            &escrow,
            &dispute_resolver,
            all_processed,
            current_balance,
            total,
            &distributions,
        )?;

        dispute_resolver.require_auth();

        let fee_result = FeeCalculator::calculate_standard_fees(total, escrow.platform_fee)?;

        let net_dists = calculate_and_distribute_fees(
            e,
            &token_client,
            &contract_address,
            &trustless_work_address,
            &escrow.roles.platform,
            &fee_result,
            &distributions,
            total,
        )?;

        e.storage().persistent().set(&DataKey::Escrow, &escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        e.storage().persistent().remove(&DataKey::Reentrancy);

        Ok((escrow, fee_result, net_dists))
    }

    pub fn resolve_dispute(
        e: &Env,
        dispute_resolver: Address,
        trustless_work_address: Address,
        milestone_indices: Vec<u32>,
        distributions: Map<Address, i128>,
    ) -> Result<(Escrow, StandardFeeResult, Vec<(Address, i128)>), EscrowError> {
        let mut escrow = EscrowManager::get_escrow(e)?;
        let contract_address = e.current_contract_address();

        let token_client = TokenClient::new(&e, &escrow.trustline.address);
        let current_balance = token_client.balance(&contract_address);

        let mut total: i128 = 0;
        for (_addr, amount) in distributions.iter() {
            if amount <= 0 {
                return Err(EscrowError::AmountsToBeTransferredShouldBePositive);
            }
            total = BasicMath::safe_add(total, amount)?;
        }

        validate_dispute_resolution_conditions(
            &escrow,
            &dispute_resolver,
            current_balance,
            total,
            &distributions,
            &milestone_indices,
        )?;

        dispute_resolver.require_auth();

        // Effects before interactions: only resolve the specified disputed milestones
        for index in milestone_indices.iter() {
            let mut milestone = escrow.milestones.get(index).unwrap();
            milestone.dispute.resolved = true;
            milestone.dispute.is_disputed = false;
            escrow.milestones.set(index, milestone);
        }
        e.storage().persistent().set(&DataKey::Escrow, &escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        let fee_result = FeeCalculator::calculate_standard_fees(total, escrow.platform_fee)?;

        let net_dists = calculate_and_distribute_fees(
            e,
            &token_client,
            &contract_address,
            &trustless_work_address,
            &escrow.roles.platform,
            &fee_result,
            &distributions,
            total,
        )?;

        Ok((escrow, fee_result, net_dists))
    }

    pub fn dispute_milestones(
        e: &Env,
        signer: Address,
        milestone_indices: Vec<u32>,
        reason: String,
    ) -> Result<Escrow, EscrowError> {
        const MAX_REASON_LEN: u32 = 500;
        if reason.len() > MAX_REASON_LEN {
            return Err(EscrowError::StringTooLong);
        }

        let mut escrow = EscrowManager::get_escrow(e)?;
        validate_batch_milestone_dispute_conditions(&escrow, &signer, &milestone_indices)?;

        signer.require_auth();

        for index in milestone_indices.iter() {
            let mut milestone = escrow.milestones.get(index).unwrap();
            milestone.dispute.is_disputed = true;
            milestone.dispute.reason = reason.clone();
            escrow.milestones.set(index, milestone);
        }

        e.storage().persistent().set(&DataKey::Escrow, &escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        Ok(escrow)
    }
}
