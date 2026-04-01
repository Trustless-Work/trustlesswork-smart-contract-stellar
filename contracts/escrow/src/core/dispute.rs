use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Map, String};

use crate::core::escrow::EscrowManager;
use crate::core::validators::dispute::{validate_withdraw_remaining_funds_conditions};
use crate::error::EscrowError;
use crate::modules::fee::distribution::calculate_and_distribute_fees;
use crate::modules::{
    fee::{FeeCalculator, FeeCalculatorTrait},
    math::{BasicArithmetic, BasicMath},
};
use crate::storage::types::{DataKey, Escrow};

use super::validators::dispute::{
    validate_dispute_flag_change_conditions, validate_dispute_resolution_conditions,
};

pub struct DisputeManager;

impl DisputeManager {
    pub fn withdraw_remaining_funds(
        e: &Env,
        dispute_resolver: Address,
        trustless_work_address: Address,
        distributions: Map<Address, i128>,
    ) -> Result<Escrow, EscrowError> {
        let escrow = EscrowManager::get_escrow(e)?;
        let contract_address = e.current_contract_address();

        let all_processed = escrow.released || escrow.dispute.resolved || escrow.dispute.is_disputed;

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
            &distributions
        )?;

        dispute_resolver.require_auth();

        let fee_result = FeeCalculator::calculate_standard_fees(total, escrow.platform_fee)?;

        calculate_and_distribute_fees(
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

        Ok(escrow)
    }

    pub fn resolve_dispute(
        e: &Env,
        dispute_resolver: Address,
        trustless_work_address: Address,
        distributions: Map<Address, i128>,
    ) -> Result<Escrow, EscrowError> {
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
            &distributions
        )?;

        dispute_resolver.require_auth();

        let fee_result = FeeCalculator::calculate_standard_fees(total, escrow.platform_fee)?;

        calculate_and_distribute_fees(
            e,
            &token_client,
            &contract_address,
            &trustless_work_address,
            &escrow.roles.platform,
            &fee_result,
            &distributions,
            total,
        )?;

        escrow.dispute.resolved = true;
        escrow.dispute.is_disputed = false;
        e.storage().persistent().set(&DataKey::Escrow, &escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        Ok(escrow)
    }

    pub fn dispute_escrow(e: &Env, signer: Address, reason: String) -> Result<Escrow, EscrowError> {
        let mut escrow = EscrowManager::get_escrow(e)?;
        validate_dispute_flag_change_conditions(&escrow, &signer)?;

        signer.require_auth();

        escrow.dispute.is_disputed = true;
        escrow.dispute.reason = reason;
        e.storage().persistent().set(&DataKey::Escrow, &escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        Ok(escrow)
    }
}
