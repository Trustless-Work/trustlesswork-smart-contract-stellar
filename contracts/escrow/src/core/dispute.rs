use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Map, String};

use crate::core::escrow::EscrowManager;
use crate::core::validators::milestone::validate_and_convert_milestone_index;
use crate::error::ContractError;
use crate::modules::{
    fee::{FeeCalculator, FeeCalculatorTrait},
    math::{BasicArithmetic, BasicMath},
};
use crate::storage::types::{DataKey, Escrow, Milestone};

use super::validators::dispute::{
    validate_dispute_flag_change_conditions, validate_dispute_resolution_conditions,
    validate_withdraw_remaining_funds_conditions,
};

pub struct DisputeManager;

impl DisputeManager {
    pub fn withdraw_remaining_funds(
        e: &Env,
        dispute_resolver: Address,
        trustless_work_address: Address,
        distributions: Map<Address, i128>,
    ) -> Result<Escrow, ContractError> {
        let escrow = EscrowManager::get_escrow(e)?;
        let contract_address = e.current_contract_address();

        let mut all_processed = true;
        for m in escrow.milestones.iter() {
            let flags = &m.flags;
            if !(flags.released || flags.resolved || flags.disputed) {
                all_processed = false;
                break;
            }
        }

        let token_client = TokenClient::new(&e, &escrow.trustline.address);
        let current_balance = token_client.balance(&contract_address);
        let mut total: i128 = 0;
        for (_addr, amount) in distributions.iter() {
            if amount <= 0 {
                return Err(ContractError::AmountsToBeTransferredShouldBePositive);
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

        let fee_result =
            FeeCalculator::calculate_standard_fees(total, escrow.platform_fee)?;
        let total_fees = BasicMath::safe_add(fee_result.trustless_work_fee, fee_result.platform_fee)?;

        if fee_result.trustless_work_fee > 0 {
            token_client.transfer(&contract_address, &trustless_work_address, &fee_result.trustless_work_fee);
        }
        if fee_result.platform_fee > 0 {
            token_client.transfer(
                &contract_address,
                &escrow.roles.platform_address,
                &fee_result.platform_fee,
            );
        }

        for (addr, amount) in distributions.iter() {
            if amount > 0 {
                let fee_share =
                    BasicMath::safe_div(BasicMath::safe_mul(amount, total_fees)?, total)?;
                let net_amount = BasicMath::safe_sub(amount, fee_share)?;
                if net_amount > 0 {
                    token_client.transfer(&contract_address, &addr, &net_amount);
                }
            }
        }

        e.storage().persistent().set(&DataKey::Escrow, &escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        Ok(escrow)
    }

    pub fn resolve_milestone_dispute(
        e: &Env,
        dispute_resolver: Address,
        milestone_index: u32,
        trustless_work_address: Address,
        distributions: Map<Address, i128>,
    ) -> Result<Escrow, ContractError> {
        let mut escrow = EscrowManager::get_escrow(e)?;
        let contract_address = e.current_contract_address();

        let token_client = TokenClient::new(&e, &escrow.trustline.address);
        let current_balance = token_client.balance(&contract_address);

        let milestones = escrow.milestones.clone();
        let milestone = match milestones.get(milestone_index) {
            Some(m) => m,
            None => return Err(ContractError::InvalidMileStoneIndex),
        };

        let mut total: i128 = 0;
        for (_addr, amount) in distributions.iter() {
            if amount <= 0 {
                return Err(ContractError::AmountsToBeTransferredShouldBePositive);
            }
            total = BasicMath::safe_add(total, amount)?;
        }

        validate_dispute_resolution_conditions(
            &escrow,
            &milestone,
            &dispute_resolver,
            current_balance,
            total,
            &distributions,
        )?;

        dispute_resolver.require_auth();

        let fee_result =
            FeeCalculator::calculate_standard_fees(total, escrow.platform_fee)?;
        let total_fees = BasicMath::safe_add(fee_result.trustless_work_fee, fee_result.platform_fee)?;

        if fee_result.trustless_work_fee > 0 {
            token_client.transfer(&contract_address, &trustless_work_address, &fee_result.trustless_work_fee);
        }
        if fee_result.platform_fee > 0 {
            token_client.transfer(
                &contract_address,
                &escrow.roles.platform_address,
                &fee_result.platform_fee,
            );
        }

        for (addr, amount) in distributions.iter() {
            if amount > 0 {
                let fee_share = BasicMath::safe_div(BasicMath::safe_mul(amount, total_fees)?, total)?;
                let net_amount = BasicMath::safe_sub(amount, fee_share)?;
                if net_amount > 0 {
                    token_client.transfer(&contract_address, &addr, &net_amount);
                }
            }
        }

        let mut updated_milestones = escrow.milestones.clone();
        let mut new_flags = milestone.flags.clone();
        new_flags.resolved = true;
        new_flags.disputed = false;
        updated_milestones.set(
            milestone_index,
            Milestone {
                status: String::from_str(&e, "resolved"),
                flags: new_flags,
                ..milestone.clone()
            },
        );
        escrow.milestones = updated_milestones;

        e.storage().persistent().set(&DataKey::Escrow, &escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        Ok(escrow)
    }

    pub fn dispute_milestone(
        e: &Env,
        milestone_index: u32,
        signer: Address,
    ) -> Result<Escrow, ContractError> {
        let mut escrow = EscrowManager::get_escrow(e)?;
        validate_dispute_flag_change_conditions(&escrow, milestone_index, &signer)?;

        signer.require_auth();

        let idx = validate_and_convert_milestone_index(
            milestone_index,
            escrow.milestones.len(),
        )?;

        let mut target = escrow
            .milestones
            .get(idx)
            .ok_or(ContractError::InvalidMileStoneIndex)?;
        target.flags.disputed = true;
        escrow.milestones.set(idx, target);
        e.storage().persistent().set(&DataKey::Escrow, &escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        Ok(escrow)
    }
}
