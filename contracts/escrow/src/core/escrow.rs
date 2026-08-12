use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Symbol, Vec};

use crate::core::validators::escrow::{
    validate_escrow_property_change_conditions, validate_fund_escrow_conditions,
    validate_initialize_escrow_conditions, validate_manage_milestones_conditions,
    validate_release_milestones_conditions,
};
use crate::error::{EscrowError, ReleaseError};
use crate::modules::fee::{FeeCalculator, FeeCalculatorTrait};
use crate::modules::math::{BasicArithmetic, BasicMath};
use crate::storage::types::{
    AddressBalance, DataKey, Escrow, Milestone, MilestonePayout, MilestoneUpdate,
};

pub struct EscrowManager;

impl EscrowManager {
    #[inline]
    pub fn get_receiver(milestone: &Milestone) -> Address {
        milestone.receiver.clone()
    }

    pub fn initialize_escrow(e: &Env, escrow_properties: Escrow) -> Result<Escrow, EscrowError> {
        validate_initialize_escrow_conditions(e, &escrow_properties)?;
        let stored_admin: Address = e
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(EscrowError::OnlyAdminAddressExecuteThisFunction)?;
        stored_admin.require_auth();
        e.storage()
            .persistent()
            .set(&DataKey::Escrow, &escrow_properties);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);
        e.storage().persistent().remove(&DataKey::Admin);
        e.storage().persistent().remove(&DataKey::ApprovedWasmHash);
        Ok(escrow_properties)
    }

    pub fn fund_escrow(
        e: &Env,
        signer: &Address,
        expected_escrow: &Escrow,
        amount: i128,
    ) -> Result<i128, EscrowError> {
        let stored_escrow: Escrow = Self::get_escrow(e)?;
        let token_client = TokenClient::new(e, &stored_escrow.trustline.address);
        let balance = token_client.balance(signer);
        validate_fund_escrow_conditions(amount, balance, &stored_escrow, expected_escrow)?;

        signer.require_auth();

        token_client.transfer(signer, &e.current_contract_address(), &amount);

        let current_funded: i128 = e
            .storage()
            .persistent()
            .get(&DataKey::FundedAmount)
            .unwrap_or(0);
        let new_funded = BasicMath::safe_add(current_funded, amount)?;
        e.storage()
            .persistent()
            .set(&DataKey::FundedAmount, &new_funded);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::FundedAmount, 17280, 31536000);

        Ok(new_funded)
    }

    pub fn release_funds(
        e: &Env,
        release_signer: &Address,
        trustless_work_address: &Address,
        milestone_indices: Vec<u32>,
    ) -> Result<Vec<MilestonePayout>, ReleaseError> {
        let escrow = Self::get_escrow(e).map_err(|_| ReleaseError::EscrowNotFound)?;
        validate_release_milestones_conditions(&escrow, release_signer, &milestone_indices)?;
        release_signer.require_auth();
        Self::release_funds_execute(e, trustless_work_address, milestone_indices, escrow)
    }

    pub(crate) fn release_funds_inner(
        e: &Env,
        release_signer: &Address,
        trustless_work_address: &Address,
        milestone_indices: Vec<u32>,
    ) -> Result<Vec<MilestonePayout>, ReleaseError> {
        let escrow = Self::get_escrow(e).map_err(|_| ReleaseError::EscrowNotFound)?;
        validate_release_milestones_conditions(&escrow, release_signer, &milestone_indices)?;
        Self::release_funds_execute(e, trustless_work_address, milestone_indices, escrow)
    }

    fn release_funds_execute(
        e: &Env,
        trustless_work_address: &Address,
        milestone_indices: Vec<u32>,
        mut escrow: Escrow,
    ) -> Result<Vec<MilestonePayout>, ReleaseError> {
        let mut total_amount: i128 = 0;
        for index in milestone_indices.iter() {
            let milestone = escrow.milestones.get(index).unwrap();
            // safe_add can only fail with Overflow, so map straight to it.
            total_amount = BasicMath::safe_add(total_amount, milestone.amount)
                .map_err(|_| ReleaseError::Overflow)?;
        }

        let contract_address = e.current_contract_address();
        let token_client = TokenClient::new(e, &escrow.trustline.address);

        if token_client.balance(&contract_address) < total_amount {
            return Err(ReleaseError::EscrowBalanceNotEnoughToSendEarnings);
        }

        // Effects before interactions: commit state before any external transfer
        for index in milestone_indices.iter() {
            let mut milestone = escrow.milestones.get(index).unwrap();
            milestone.released = true;
            escrow.milestones.set(index, milestone);
        }

        e.storage().persistent().set(&DataKey::Escrow, &escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        let mut payouts: Vec<MilestonePayout> = Vec::new(e);
        for index in milestone_indices.iter() {
            let milestone = escrow.milestones.get(index).unwrap();
            let fee_result =
                FeeCalculator::calculate_standard_fees(milestone.amount, escrow.platform_fee)
                    .map_err(|e| match e {
                        EscrowError::Overflow => ReleaseError::Overflow,
                        EscrowError::Underflow => ReleaseError::Underflow,
                        _ => ReleaseError::DivisionError,
                    })?;

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

            let receiver = Self::get_receiver(&milestone);
            if fee_result.receiver_amount > 0 {
                token_client.transfer(&contract_address, &receiver, &fee_result.receiver_amount);
            }

            payouts.push_back(MilestonePayout {
                index,
                receiver,
                amount: milestone.amount,
                platform_fee: fee_result.platform_fee,
                trustless_work_fee: fee_result.trustless_work_fee,
                net_amount: fee_result.receiver_amount,
            });
        }

        Ok(payouts)
    }

    pub fn change_escrow_properties(
        e: &Env,
        admin: &Address,
        escrow_properties: Escrow,
    ) -> Result<Escrow, EscrowError> {
        let existing_escrow = Self::get_escrow(e)?;
        let funded_amount: i128 = e
            .storage()
            .persistent()
            .get(&DataKey::FundedAmount)
            .unwrap_or(0);

        validate_escrow_property_change_conditions(
            &existing_escrow,
            &escrow_properties,
            admin,
            funded_amount,
        )?;

        admin.require_auth();

        let mut escrow_to_save = escrow_properties;
        escrow_to_save.milestones = existing_escrow.milestones.clone();

        e.storage()
            .persistent()
            .set(&DataKey::Escrow, &escrow_to_save);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);
        Ok(escrow_to_save)
    }

    pub fn manage_milestones(
        e: &Env,
        admin: &Address,
        new_milestones: Vec<Milestone>,
        milestone_updates: Vec<MilestoneUpdate>,
    ) -> Result<Escrow, EscrowError> {
        let mut existing_escrow = Self::get_escrow(e)?;
        let funded_amount: i128 = e
            .storage()
            .persistent()
            .get(&DataKey::FundedAmount)
            .unwrap_or(0);

        validate_manage_milestones_conditions(
            &existing_escrow,
            admin,
            &new_milestones,
            &milestone_updates,
            funded_amount,
        )?;
        admin.require_auth();

        for update in milestone_updates.iter() {
            let mut milestone = existing_escrow.milestones.get(update.index).unwrap();
            if let Some(desc) = update.new_description {
                milestone.description = desc;
            }
            if let Some(amount) = update.new_amount {
                milestone.amount = amount;
            }
            existing_escrow.milestones.set(update.index, milestone);
        }

        for milestone in new_milestones.iter() {
            existing_escrow.milestones.push_back(milestone);
        }

        e.storage()
            .persistent()
            .set(&DataKey::Escrow, &existing_escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);
        Ok(existing_escrow)
    }

    pub fn get_multiple_escrow_balances(
        e: &Env,
        addresses: Vec<Address>,
    ) -> Result<Vec<AddressBalance>, EscrowError> {
        const MAX_ESCROWS: u32 = 20;
        if addresses.len() > MAX_ESCROWS {
            return Err(EscrowError::TooManyEscrowsRequested);
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
    ) -> Result<Escrow, EscrowError> {
        Ok(e.invoke_contract::<Escrow>(contract_id, &Symbol::new(e, "get_escrow"), Vec::new(e)))
    }

    pub fn get_escrow(e: &Env) -> Result<Escrow, EscrowError> {
        Ok(e.storage()
            .persistent()
            .get(&DataKey::Escrow)
            .ok_or(EscrowError::EscrowNotFound)?)
    }
}
