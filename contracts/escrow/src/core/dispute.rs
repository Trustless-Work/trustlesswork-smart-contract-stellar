use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Map};

use crate::core::escrow::EscrowManager;
use crate::core::validators::dispute::validate_withdraw_remaining_funds_conditions;
use crate::error::ContractError;
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
    ) -> Result<Escrow, ContractError> {
        dispute_resolver.require_auth();
        let escrow = EscrowManager::get_escrow(e)?;
        let contract_address = e.current_contract_address();

        let mut all_processed = true;
        let flags = &escrow.flags;
        if !(flags.released || flags.resolved || flags.disputed) {
            all_processed = false;
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
            total
        )?;

        let fee_result = FeeCalculator::calculate_standard_fees(total, escrow.platform_fee)?;
        let total_fees =
            BasicMath::safe_add(fee_result.trustless_work_fee, fee_result.platform_fee)?;

        if fee_result.trustless_work_fee > 0 {
            token_client.transfer(
                &contract_address,
                &trustless_work_address,
                &fee_result.trustless_work_fee,
            );
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
                let fee_share = BasicMath::safe_div(
                    BasicMath::safe_mul(amount, total_fees)?,
                    total,
                )?;
                let net_amount = BasicMath::safe_sub(amount, fee_share)?;
                if net_amount > 0 {
                    token_client.transfer(&contract_address, &addr, &net_amount);
                }
            }
        }

        e.storage().instance().set(&DataKey::Escrow, &escrow);

        Ok(escrow)
    }

    pub fn resolve_dispute(
        e: &Env,
        dispute_resolver: Address,
        trustless_work_address: Address,
        distributions: Map<Address, i128>,
    ) -> Result<Escrow, ContractError> {
        dispute_resolver.require_auth();
        let mut escrow = EscrowManager::get_escrow(e)?;
        let contract_address = e.current_contract_address();

        let token_client = TokenClient::new(&e, &escrow.trustline.address);
        let current_balance = token_client.balance(&contract_address);

        let mut total: i128 = 0;
        for (_addr, amount) in distributions.iter() {
            if amount <= 0 {
                return Err(ContractError::AmountsToBeTransferredShouldBePositive);
            }
            total = BasicMath::safe_add(total, amount)?;
        }

        validate_dispute_resolution_conditions(
            &escrow,
            &dispute_resolver,
            current_balance,
            total,
        )?;

        let fee_result = FeeCalculator::calculate_standard_fees(total, escrow.platform_fee)?;
        let total_fees =
            BasicMath::safe_add(fee_result.trustless_work_fee, fee_result.platform_fee)?;

        if fee_result.trustless_work_fee > 0 {
            token_client.transfer(
                &contract_address,
                &trustless_work_address,
                &fee_result.trustless_work_fee,
            );
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
                let fee_share = BasicMath::safe_div(
                BasicMath::safe_mul(amount, total_fees)?,
                total,
                )?;
                let net_amount = BasicMath::safe_sub(amount, fee_share)?;
                if net_amount > 0 {
                    token_client.transfer(&contract_address, &addr, &net_amount);
                }
            }
        }

        escrow.flags.resolved = true;
        escrow.flags.disputed = false;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        Ok(escrow)
    }

    pub fn dispute_escrow(e: &Env, signer: Address) -> Result<Escrow, ContractError> {
        signer.require_auth();
        let mut escrow = EscrowManager::get_escrow(e)?;
        validate_dispute_flag_change_conditions(&escrow, &signer)?;

        escrow.flags.disputed = true;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        Ok(escrow)
    }
}
