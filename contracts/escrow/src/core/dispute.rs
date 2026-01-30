use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Map, Vec};

use crate::core::escrow::EscrowManager;
use crate::core::validators::dispute::{
    validate_distributions_size, validate_withdraw_remaining_funds_conditions,
};
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
        validate_distributions_size(&distributions)?;

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
            total,
        )?;

        Self::execute_distribution(
            e,
            &escrow,
            distributions,
            trustless_work_address,
            total,
            &token_client,
            &contract_address,
        )?;

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
        validate_distributions_size(&distributions)?;

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

        Self::execute_distribution(
            e,
            &escrow,
            distributions,
            trustless_work_address,
            total,
            &token_client,
            &contract_address,
        )?;

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

    fn execute_distribution(
        e: &Env,
        escrow: &Escrow,
        distributions: Map<Address, i128>,
        trustless_work_address: Address,
        total: i128,
        token_client: &TokenClient,
        contract_address: &Address,
    ) -> Result<(), ContractError> {
        let fee_result = FeeCalculator::calculate_standard_fees(total, escrow.platform_fee)?;

        let mut actual_trustless_fees = 0i128;
        let mut actual_platform_fees = 0i128;
        let mut net_distributions: Vec<(Address, i128)> = Vec::new(e);

        for (addr, amount) in distributions.iter() {
            if amount <= 0 {
                continue;
            }

            let recipient_trustless_fee =
                BasicMath::safe_mul(amount, fee_result.trustless_work_fee)?
                    .checked_div(total)
                    .ok_or(ContractError::DivisionError)?;
            let recipient_platform_fee = BasicMath::safe_mul(amount, fee_result.platform_fee)?
                .checked_div(total)
                .ok_or(ContractError::DivisionError)?;

            let total_recipient_fee =
                BasicMath::safe_add(recipient_trustless_fee, recipient_platform_fee)?;
            let net_amount = BasicMath::safe_sub(amount, total_recipient_fee)?;

            actual_trustless_fees =
                BasicMath::safe_add(actual_trustless_fees, recipient_trustless_fee)?;
            actual_platform_fees =
                BasicMath::safe_add(actual_platform_fees, recipient_platform_fee)?;

            if net_amount > 0 {
                net_distributions.push_back((addr.clone(), net_amount));
            }
        }

        if actual_trustless_fees > 0 {
            token_client.transfer(
                contract_address,
                &trustless_work_address,
                &actual_trustless_fees,
            );
        }
        if actual_platform_fees > 0 {
            token_client.transfer(
                contract_address,
                &escrow.roles.platform_address,
                &actual_platform_fees,
            );
        }

        for (addr, net_amount) in net_distributions.iter() {
            token_client.transfer(contract_address, &addr, &net_amount);
        }

        Ok(())
    }
}
