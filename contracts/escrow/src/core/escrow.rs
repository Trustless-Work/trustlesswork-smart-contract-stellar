use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Symbol, Vec};

use crate::core::validators::escrow::{
    validate_escrow_property_change_conditions, validate_fund_escrow_conditions,
    validate_initialize_escrow_conditions, validate_release_conditions,
};
use crate::error::ContractError;
use crate::modules::fee::{FeeCalculator, FeeCalculatorTrait};
use crate::modules::math::{BasicArithmetic, BasicMath};
use crate::modules::vault::VaultManager;
use crate::storage::types::{AddressBalance, DataKey, Escrow, VaultPosition};

pub struct EscrowManager;

impl EscrowManager {
    #[inline]
    pub fn get_receiver(escrow: &Escrow) -> Address {
        escrow.roles.receiver.clone()
    }

    pub fn initialize_escrow(e: &Env, escrow_properties: Escrow) -> Result<Escrow, ContractError> {
        validate_initialize_escrow_conditions(e, escrow_properties.clone())?;
        if let Some(vault) = &escrow_properties.vault {
            VaultManager::validate_asset(e, vault, &escrow_properties.trustline)?;
        }
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
        signer: &Address,
        expected_escrow: &Escrow,
        amount: i128,
    ) -> Result<(), ContractError> {
        let stored_escrow: Escrow = Self::get_escrow(e)?;
        let token_client = TokenClient::new(e, &stored_escrow.trustline.address);
        let balance = token_client.balance(signer);
        validate_fund_escrow_conditions(amount, balance, &stored_escrow, expected_escrow)?;

        signer.require_auth();

        token_client.transfer(signer, &e.current_contract_address(), &amount);

        // Once released or resolved the escrow only pays out, so new funds stay idle.
        if let Some(vault) = &stored_escrow.vault {
            if !stored_escrow.flags.released && !stored_escrow.flags.resolved {
                VaultManager::deposit(e, vault, &stored_escrow.trustline.address, amount);
            }
        }

        let funded: i128 = e
            .storage()
            .persistent()
            .get(&DataKey::FundedAmount)
            .unwrap_or(0);
        let new_funded = BasicMath::safe_add(funded, amount)?;
        e.storage()
            .persistent()
            .set(&DataKey::FundedAmount, &new_funded);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::FundedAmount, 17280, 31536000);
        Ok(())
    }

    pub fn release_funds(
        e: &Env,
        release_signer: &Address,
        trustless_work_address: &Address,
    ) -> Result<(), ContractError> {
        let mut escrow = Self::get_escrow(e)?;
        validate_release_conditions(&escrow, release_signer)?;

        release_signer.require_auth();

        escrow.flags.released = true;
        e.storage().persistent().set(&DataKey::Escrow, &escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        let contract_address = e.current_contract_address();
        let token_client = TokenClient::new(e, &escrow.trustline.address);

        // With a vault the receiver gets everything redeemed, yield included.
        let payout = match &escrow.vault {
            Some(vault) => {
                let funded: i128 = e
                    .storage()
                    .persistent()
                    .get(&DataKey::FundedAmount)
                    .unwrap_or(0);
                if funded < escrow.amount {
                    return Err(ContractError::EscrowBalanceNotEnoughToSendEarnings);
                }
                VaultManager::redeem_all(e, vault);
                token_client.balance(&contract_address)
            }
            None => {
                if token_client.balance(&contract_address) < escrow.amount {
                    return Err(ContractError::EscrowBalanceNotEnoughToSendEarnings);
                }
                escrow.amount
            }
        };

        let fee_result = FeeCalculator::calculate_standard_fees(payout, escrow.platform_fee)?;

        if fee_result.trustless_work_fee > 0 {
            token_client.transfer(
                &contract_address,
                trustless_work_address,
                &fee_result.trustless_work_fee,
            );
        }

        if fee_result.platform_fee > 0 {
            token_client.transfer(
            &contract_address,
            &escrow.roles.platform,
            &fee_result.platform_fee,
            );
        }

        let receiver = Self::get_receiver(&escrow);
        if fee_result.receiver_amount > 0 {
            token_client.transfer(&contract_address, &receiver, &fee_result.receiver_amount);
        }

        Ok(())
    }
    pub fn change_escrow_properties(
        e: &Env,
        platform: &Address,
        escrow_properties: Escrow,
    ) -> Result<Escrow, ContractError> {
        let existing_escrow = Self::get_escrow(e)?;
        let token_client = TokenClient::new(e, &existing_escrow.trustline.address);
        let contract_balance = Self::total_balance(e, &existing_escrow, &token_client)?;

        validate_escrow_property_change_conditions(
            &existing_escrow,
            &escrow_properties,
            platform,
            contract_balance,
        )?;

        platform.require_auth();

        if let Some(vault) = &escrow_properties.vault {
            VaultManager::validate_asset(e, vault, &escrow_properties.trustline)?;
        }

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

        let mut balances: Vec<AddressBalance> = Vec::new(e);
        let self_addr = e.current_contract_address();
        for address in addresses.iter() {
            let escrow = if address == self_addr {
                Self::get_escrow(e)?
            } else {
                Self::get_escrow_by_contract_id(e, &address)?
            };
            let token_client = TokenClient::new(e, &escrow.trustline.address);
            let mut balance = token_client.balance(&address);
            if let Some(vault) = &escrow.vault {
                let (_, underlying) = VaultManager::position(e, vault, &address);
                balance = BasicMath::safe_add(balance, underlying)?;
            }
            balances.push_back(AddressBalance {
                address: address.clone(),
                balance,
                trustline_decimals: token_client.decimals(),
            });
        }
        Ok(balances)
    }

    /// Idle balance plus the value of the escrow's vault shares.
    fn total_balance(
        e: &Env,
        escrow: &Escrow,
        token_client: &TokenClient,
    ) -> Result<i128, ContractError> {
        let contract_address = e.current_contract_address();
        let idle = token_client.balance(&contract_address);
        match &escrow.vault {
            Some(vault) => {
                let (_, underlying) = VaultManager::position(e, vault, &contract_address);
                BasicMath::safe_add(idle, underlying)
            }
            None => Ok(idle),
        }
    }

    pub fn get_vault_position(e: &Env) -> Result<Option<VaultPosition>, ContractError> {
        let escrow = Self::get_escrow(e)?;
        Ok(escrow.vault.map(|vault| {
            let (shares, underlying) =
                VaultManager::position(e, &vault, &e.current_contract_address());
            VaultPosition {
                vault,
                shares,
                underlying,
            }
        }))
    }

    pub fn get_escrow_by_contract_id(
        e: &Env,
        contract_id: &Address,
    ) -> Result<Escrow, ContractError> {
        Ok(e.invoke_contract::<Escrow>(contract_id, &Symbol::new(e, "get_escrow"), Vec::new(e)))
    }

    pub fn get_escrow(e: &Env) -> Result<Escrow, ContractError> {
        Ok(e.storage()
            .persistent()
            .get(&DataKey::Escrow)
            .ok_or(ContractError::EscrowNotFound)?)
    }
}
