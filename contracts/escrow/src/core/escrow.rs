use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Symbol, Vec};

use super::validators::escrow::{
    validate_escrow_property_change_conditions, validate_fund_escrow_conditions,
    validate_initialize_escrow_conditions, validate_release_conditions,
};
use crate::error::ContractError;
use crate::modules::fee::{FeeCalculator, FeeCalculatorTrait};
use crate::storage::types::{AddressBalance, DataKey, Escrow, Milestone};

pub struct EscrowManager;

impl EscrowManager {
    #[inline]
    pub fn get_receiver(milestone: &Milestone) -> Address {
        milestone.receiver.clone()
    }

    pub fn initialize_escrow(e: &Env, escrow_properties: Escrow) -> Result<Escrow, ContractError> {
        let token_client = TokenClient::new(&e, &escrow_properties.trustline.address);
        let escrow_balance = token_client.balance(&e.current_contract_address());
        validate_initialize_escrow_conditions(e, escrow_properties.clone(), escrow_balance)?;
        e.storage()
            .persistent()
            .set(&DataKey::Escrow, &escrow_properties);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);
        Ok(escrow_properties)
    }

    pub fn fund_escrow(
        e: &Env,
        signer: Address,
        expected_escrow: Escrow,
        amount: i128,
    ) -> Result<(), ContractError> {
        let stored_escrow: Escrow = Self::get_escrow(e)?;
        let token_client = TokenClient::new(&e, &stored_escrow.trustline.address);
        let balance = token_client.balance(&signer);

        validate_fund_escrow_conditions(amount, balance, &stored_escrow, &expected_escrow)?;

        signer.require_auth();

        token_client.transfer(&signer, &e.current_contract_address(), &amount);
        Ok(())
    }

    pub fn release_milestone_funds(
        e: &Env,
        release_signer: Address,
        trustless_work_address: Address,
        milestone_index: u32,
    ) -> Result<(), ContractError> {
        let mut escrow = EscrowManager::get_escrow(e)?;

        if let Some(milestone) = escrow.milestones.get(milestone_index) {
            validate_release_conditions(&escrow, &release_signer, &milestone, milestone_index)?;

            release_signer.require_auth();

            let mut to_update = milestone.clone();
            to_update.flags.released = true;
            escrow.milestones.set(milestone_index, to_update);

            e.storage().persistent().set(&DataKey::Escrow, &escrow);
            e.storage()
                .persistent()
                .extend_ttl(&DataKey::Escrow, 17280, 31536000);

            let contract_address = e.current_contract_address();
            let token_client = TokenClient::new(&e, &escrow.trustline.address);
            if token_client.balance(&contract_address) < milestone.amount {
                return Err(ContractError::EscrowBalanceNotEnoughToSendEarnings);
            }

            let fee_result =
                FeeCalculator::calculate_standard_fees(milestone.amount, escrow.platform_fee)?;

            if fee_result.trustless_work_fee > 0 {
                token_client.transfer(
                    &contract_address,
                    &trustless_work_address,
                    &fee_result.trustless_work_fee,
                );
            }

            if fee_result.platform_fee > 0 {
                token_client.transfer(&contract_address, &escrow.roles.platform_address, &fee_result.platform_fee);
            }

            let receiver = Self::get_receiver(&milestone);
            if fee_result.receiver_amount > 0 {
                token_client.transfer(&contract_address, &receiver, &fee_result.receiver_amount);
            }
        } else {
            return Err(ContractError::MilestoneNotFound);
        }

        Ok(())
    }

    pub fn change_escrow_properties(
        e: &Env,
        platform_address: Address,
        escrow_properties: Escrow,
    ) -> Result<Escrow, ContractError> {
        let escrow = EscrowManager::get_escrow(e)?;
        let token_client = TokenClient::new(&e, &escrow.trustline.address);
        let contract_balance = token_client.balance(&e.current_contract_address());

        validate_escrow_property_change_conditions(
            &escrow,
            &escrow_properties,
            &platform_address,
            contract_balance,
            escrow.milestones.clone(),
        )?;

        platform_address.require_auth();

        e.storage()
            .persistent()
            .set(&DataKey::Escrow, &escrow_properties);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);
        Ok(escrow_properties)
    }

    pub fn get_multiple_escrow_balances(
        e: &Env,
        addresses: Vec<Address>,
    ) -> Result<Vec<AddressBalance>, ContractError> {
        const MAX_ESCROWS: u32 = 20;
        if addresses.len() > MAX_ESCROWS {
            return Err(ContractError::TooManyEscrowsRequested);
        }

        let mut balances: Vec<AddressBalance> = Vec::new(&e);
        let self_addr = e.current_contract_address();
        for address in addresses.iter() {
            let escrow = if address == self_addr {
                Self::get_escrow(e)?
            } else {
                Self::get_escrow_by_contract_id(e, &address)?
            };
            let token_client = TokenClient::new(&e, &escrow.trustline.address);
            let balance = token_client.balance(&address);
            balances.push_back(AddressBalance {
                address: address.clone(),
                balance,
                trustline_decimals: token_client.decimals(),
            })
        }
        Ok(balances)
    }

    pub fn get_escrow_by_contract_id(
        e: &Env,
        contract_id: &Address,
    ) -> Result<Escrow, ContractError> {
        Ok(e.invoke_contract::<Escrow>(contract_id, &Symbol::new(&e, "get_escrow"), Vec::new(&e)))
    }

    pub fn get_escrow(e: &Env) -> Result<Escrow, ContractError> {
        e.storage()
            .persistent()
            .get(&DataKey::Escrow)
            .ok_or(ContractError::EscrowNotFound)?
    }
}
