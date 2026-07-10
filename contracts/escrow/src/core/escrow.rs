use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

use crate::core::validators::escrow::{
    validate_escrow_property_change_conditions, validate_fund_escrow_conditions,
    validate_initialize_escrow_conditions, validate_manage_milestones_conditions,
    validate_release_conditions,
};
use crate::error::{CctpError, EscrowError};
use crate::modules::cctp::release::{
    release_receiver_amount_via_cctp_forwarding, validate_destination,
};
use crate::modules::fee::{FeeCalculator, FeeCalculatorTrait, StandardFeeResult};
use crate::storage::types::{
    AddressBalance, CrossChainDestination, DataKey, Escrow, Milestone, MilestoneUpdate,
};

pub struct EscrowManager;

impl EscrowManager {
    #[inline]
    pub fn get_receiver(escrow: &Escrow) -> Address {
        escrow.roles.receiver.clone()
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
        let new_funded = current_funded + amount;
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
    ) -> Result<(Escrow, StandardFeeResult), EscrowError> {
        let escrow = Self::get_escrow(e)?;
        validate_release_conditions(&escrow, release_signer)?;
        release_signer.require_auth();
        Self::release_funds_execute(e, trustless_work_address, escrow)
    }

    pub(crate) fn release_funds_inner(
        e: &Env,
        release_signer: &Address,
        trustless_work_address: &Address,
    ) -> Result<(Escrow, StandardFeeResult), EscrowError> {
        let escrow = Self::get_escrow(e)?;
        validate_release_conditions(&escrow, release_signer)?;
        Self::release_funds_execute(e, trustless_work_address, escrow)
    }

    fn release_funds_execute(
        e: &Env,
        trustless_work_address: &Address,
        mut escrow: Escrow,
    ) -> Result<(Escrow, StandardFeeResult), EscrowError> {
        escrow.released = true;
        e.storage().persistent().set(&DataKey::Escrow, &escrow);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, 17280, 31536000);

        let contract_address = e.current_contract_address();
        let token_client = TokenClient::new(e, &escrow.trustline.address);

        if token_client.balance(&contract_address) < escrow.amount {
            return Err(EscrowError::EscrowBalanceNotEnoughToSendEarnings);
        }

        let fee_result =
            FeeCalculator::calculate_standard_fees(escrow.amount, escrow.platform_fee)?;

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
            // Route by the receiver's own registered preference.
            match Self::get_cross_chain_destination_opt(e) {
                Some(destination) => {
                    release_receiver_amount_via_cctp_forwarding(
                        e,
                        &token_client,
                        &contract_address,
                        &escrow.trustline.address,
                        fee_result.receiver_amount,
                        destination.destination_domain,
                        &destination.mint_recipient,
                        destination.max_fee,
                        &receiver,
                    );
                }
                None => {
                    token_client.transfer(
                        &contract_address,
                        &receiver,
                        &fee_result.receiver_amount,
                    );
                }
            }
        }

        Ok((escrow, fee_result))
    }

    /// Registers the receiver's cross-chain payout target. Only the receiver.
    /// `max_fee` is the Forwarding Service ceiling the receiver approves —
    /// the API sizes it from a live Circle quote when building this call, the
    /// caller of the API never supplies it directly.
    pub fn set_cross_chain_destination(
        e: &Env,
        receiver: &Address,
        destination_domain: u32,
        mint_recipient: &BytesN<32>,
        max_fee: i128,
    ) -> Result<(), CctpError> {
        let escrow = Self::assert_receiver(e, receiver)?;
        receiver.require_auth();
        validate_destination(destination_domain, mint_recipient, max_fee, escrow.amount)?;

        let destination = CrossChainDestination {
            destination_domain,
            mint_recipient: mint_recipient.clone(),
            max_fee,
        };
        e.storage()
            .persistent()
            .set(&DataKey::CrossChainDestination, &destination);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::CrossChainDestination, 17280, 31536000);
        Ok(())
    }

    /// Clears the cross-chain target, reverting to a Stellar payout.
    pub fn clear_cross_chain_destination(
        e: &Env,
        receiver: &Address,
    ) -> Result<(), CctpError> {
        Self::assert_receiver(e, receiver)?;
        receiver.require_auth();
        e.storage()
            .persistent()
            .remove(&DataKey::CrossChainDestination);
        Ok(())
    }

    pub fn get_cross_chain_destination(
        e: &Env,
    ) -> Result<CrossChainDestination, CctpError> {
        Self::get_cross_chain_destination_opt(e).ok_or(CctpError::DestinationNotSet)
    }

    #[inline]
    fn get_cross_chain_destination_opt(e: &Env) -> Option<CrossChainDestination> {
        e.storage().persistent().get(&DataKey::CrossChainDestination)
    }

    fn assert_receiver(e: &Env, receiver: &Address) -> Result<Escrow, CctpError> {
        let escrow = Self::get_escrow(e).map_err(|_| CctpError::DestinationNotSet)?;
        if *receiver != escrow.roles.receiver {
            return Err(CctpError::OnlyReceiverCanSetDestination);
        }
        Ok(escrow)
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
        escrow_to_save.dispute = existing_escrow.dispute.clone();
        escrow_to_save.released = existing_escrow.released;

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
