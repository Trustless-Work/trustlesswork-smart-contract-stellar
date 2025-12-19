use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Symbol, Vec};

use crate::core::validators::escrow::{
    validate_escrow_property_change_conditions, validate_fund_escrow_conditions,
    validate_initialize_escrow_conditions, validate_release_conditions,
};
use crate::error::ContractError;
use crate::modules::fee::{FeeCalculator, FeeCalculatorTrait};
use crate::storage::types::{AddressBalance, DataKey, Escrow};

pub struct EscrowManager;

impl EscrowManager {
    #[inline]
    pub fn get_receiver(escrow: &Escrow) -> Address {
        escrow.roles.receiver.clone()
    }

    pub fn initialize_escrow(e: &Env, escrow_properties: Escrow) -> Result<Escrow, ContractError> {
        validate_initialize_escrow_conditions(e, escrow_properties.clone())?;
        e.storage()
            .instance()
            .set(&DataKey::Escrow, &escrow_properties);
        Ok(escrow_properties)
    }

    pub fn fund_escrow(
        e: &Env,
        signer: &Address,
        expected_escrow: &Escrow,
        amount: i128,
    ) -> Result<(), ContractError> {
        let stored_escrow: Escrow = Self::get_escrow(e)?;
        
        signer.require_auth();
        let token_client = TokenClient::new(e, &stored_escrow.trustline.address);
        let balance = token_client.balance(signer);
        validate_fund_escrow_conditions(amount, balance, &stored_escrow, expected_escrow)?;

        token_client.transfer(signer, &e.current_contract_address(), &amount);
        Ok(())
    }

    pub fn release_funds(
        e: &Env,
        release_signer: &Address,
        trustless_work_address: &Address,
    ) -> Result<(), ContractError> {
        release_signer.require_auth();

        let mut escrow = Self::get_escrow(e)?;
        validate_release_conditions(&escrow, release_signer)?;

        escrow.flags.released = true;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        let contract_address = e.current_contract_address();
        let token_client = TokenClient::new(e, &escrow.trustline.address);

        if token_client.balance(&contract_address) < escrow.amount {
            return Err(ContractError::EscrowBalanceNotEnoughToSendEarnings);
        }

        let fee_result =
            FeeCalculator::calculate_standard_fees(escrow.amount as i128, escrow.platform_fee)?;

        token_client.transfer(
            &contract_address,
            trustless_work_address,
            &fee_result.trustless_work_fee,
        );
        token_client.transfer(
            &contract_address,
            &escrow.roles.platform_address,
            &fee_result.platform_fee,
        );

        let receiver = Self::get_receiver(&escrow);
        token_client.transfer(&contract_address, &receiver, &fee_result.receiver_amount);

        Ok(())
    }
    pub fn change_escrow_properties(
        e: &Env,
        platform_address: &Address,
        escrow_properties: Escrow,
    ) -> Result<Escrow, ContractError> {
        platform_address.require_auth();
        let existing_escrow = Self::get_escrow(e)?;
        let token_client = TokenClient::new(e, &existing_escrow.trustline.address);
        let contract_balance = token_client.balance(&e.current_contract_address());

        validate_escrow_property_change_conditions(
            &existing_escrow,
            &escrow_properties,
            platform_address,
            contract_balance,
        )?;

        e.storage()
            .instance()
            .set(&DataKey::Escrow, &escrow_properties);
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

        let mut balances: Vec<AddressBalance> = Vec::new(e);
        let self_addr = e.current_contract_address();
        for address in addresses.iter() {
            let escrow = if address == self_addr {
                Self::get_escrow(e)?
            } else {
                Self::get_escrow_by_contract_id(e, &address)?
            };
            let token_client = TokenClient::new(e, &escrow.trustline.address);
            let balance = token_client.balance(&address);
            balances.push_back(AddressBalance {
                address: address.clone(),
                balance,
                trustline_decimals: token_client.decimals(),
            });
        }
        Ok(balances)
    }

    pub fn get_escrow_by_contract_id(
        e: &Env,
        contract_id: &Address,
    ) -> Result<Escrow, ContractError> {
        Ok(e.invoke_contract::<Escrow>(contract_id, &Symbol::new(e, "get_escrow"), Vec::new(e)))
    }

    pub fn get_escrow(e: &Env) -> Result<Escrow, ContractError> {
        Ok(e.storage()
            .instance()
            .get(&DataKey::Escrow)
            .ok_or(ContractError::EscrowNotFound)?)
    }
}
