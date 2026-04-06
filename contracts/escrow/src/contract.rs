use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, String, Symbol, Val, Vec};

use crate::core::{DisputeManager, EscrowManager, MilestoneManager};
use crate::error::{EscrowError, MilestoneError};
use crate::events::handler::{
    ChgEsc, DisEsc, DisputeResolved, EscrowDisputed, ExtTtlEvt, FundEsc, InitEsc,
    MilestonesManaged, MilestonesApproved, MilestoneStatusChanged,
};
use crate::storage::types::{AddressBalance, DataKey, Escrow, Milestone, MilestoneStatusUpdate, MilestoneUpdate};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn __constructor(e: Env, admin: Address) {
        e.storage().persistent().set(&DataKey::Admin, &admin);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Admin, 17280, 31536000);
    }

    pub fn tw_new_single_release_escrow(
        env: Env,
        signer: Address,
        wasm_hash: BytesN<32>,
        salt: BytesN<32>,
        init_fn: Symbol,
        init_args: Vec<Val>,
        constructor_args: Vec<Val>,
    ) -> Result<(Address, Val), EscrowError> {
        if EscrowManager::get_escrow(&env).is_ok() {
            return Err(EscrowError::EscrowAlreadyInitialized);
        }

        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(EscrowError::OnlyAdminAddressExecuteThisFunction)?;

        if signer != stored_admin {
            return Err(EscrowError::OnlyAdminAddressExecuteThisFunction);
        }

        signer.require_auth();

        let deployer = env.current_contract_address();
        let deployed_address = env
            .deployer()
            .with_address(deployer, salt)
            .deploy_v2(wasm_hash, constructor_args);

        let res: Val = env.invoke_contract(&deployed_address, &init_fn, init_args);
        Ok((deployed_address, res))
    }

    ////////////////////////
    // Escrow /////
    ////////////////////////

    pub fn initialize_escrow(e: &Env, escrow_properties: Escrow) -> Result<Escrow, EscrowError> {
        let initialized_escrow = EscrowManager::initialize_escrow(e, escrow_properties)?;
        InitEsc {
            escrow: initialized_escrow.clone(),
        }
        .publish(e);
        Ok(initialized_escrow)
    }

    pub fn fund_escrow(
        e: &Env,
        signer: Address,
        expected_escrow: Escrow,
        amount: i128,
    ) -> Result<(), EscrowError> {
        EscrowManager::fund_escrow(e, &signer, &expected_escrow, amount)?;
        FundEsc { signer, amount }.publish(e);
        Ok(())
    }

    pub fn release_funds(
        e: &Env,
        release_signer: Address,
        trustless_work_address: Address,
    ) -> Result<(), EscrowError> {
        EscrowManager::release_funds(e, &release_signer, &trustless_work_address)?;
        DisEsc { release_signer }.publish(e);
        Ok(())
    }

    pub fn update_escrow(
        e: &Env,
        admin_address: Address,
        escrow_properties: Escrow,
    ) -> Result<Escrow, EscrowError> {
        let updated_escrow = EscrowManager::change_escrow_properties(
            e,
            &admin_address,
            escrow_properties.clone(),
        )?;
        ChgEsc {
            platform: admin_address,
            engagement_id: escrow_properties.engagement_id.clone(),
            new_escrow_properties: updated_escrow.clone(),
        }
        .publish(e);
        Ok(updated_escrow)
    }

    pub fn manage_milestones(
        e: &Env,
        admin_address: Address,
        new_milestones: Vec<Milestone>,
        milestone_updates: Vec<MilestoneUpdate>,
    ) -> Result<Escrow, EscrowError> {
        let updated_escrow =
            EscrowManager::manage_milestones(e, &admin_address, new_milestones, milestone_updates)?;
        MilestonesManaged {
            escrow: updated_escrow.clone(),
        }
        .publish(e);
        Ok(updated_escrow)
    }

    pub fn get_escrow(e: &Env) -> Result<Escrow, EscrowError> {
        EscrowManager::get_escrow(e)
    }

    pub fn get_escrow_by_contract_id(
        e: &Env,
        contract_id: Address,
    ) -> Result<Escrow, EscrowError> {
        EscrowManager::get_escrow_by_contract_id(e, &contract_id)
    }

    pub fn get_multiple_escrow_balances(
        e: &Env,
        addresses: Vec<Address>,
    ) -> Result<Vec<AddressBalance>, EscrowError> {
        EscrowManager::get_multiple_escrow_balances(e, addresses)
    }

    ////////////////////////
    // Admin / TTL /////
    ////////////////////////

    pub fn extend_contract_ttl(
        e: &Env,
        admin: Address,
        ledgers_to_extend: u32,
    ) -> Result<(), EscrowError> {
        let escrow = EscrowManager::get_escrow(e)?;
        if admin != escrow.roles.admin {
            return Err(EscrowError::OnlyAdminAddressExecuteThisFunction);
        }

        admin.require_auth();

        let min_ledgers = 17280u32;
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Escrow, min_ledgers, ledgers_to_extend);

        ExtTtlEvt {
            platform: admin,
            ledgers_to_extend,
        }
        .publish(e);

        Ok(())
    }

    ////////////////////////
    // Milestones /////
    ////////////////////////

    pub fn change_milestone_status(
        e: Env,
        updates: Vec<MilestoneStatusUpdate>,
        service_provider: Address,
    ) -> Result<(), MilestoneError> {
        let escrow = MilestoneManager::change_milestone_status(&e, updates, service_provider)?;
        MilestoneStatusChanged { escrow }.publish(&e);
        Ok(())
    }

    pub fn approve_milestones(
        e: Env,
        milestone_indices: Vec<u32>,
        approver: Address,
    ) -> Result<(), MilestoneError> {
        let escrow = MilestoneManager::approve_milestones(&e, milestone_indices, approver)?;
        MilestonesApproved { escrow }.publish(&e);
        Ok(())
    }

    ////////////////////////
    // Disputes /////
    ////////////////////////

    pub fn resolve_dispute(
        e: Env,
        dispute_resolver: Address,
        trustless_work_address: Address,
        distributions: Map<Address, i128>,
    ) -> Result<(), EscrowError> {
        let escrow = DisputeManager::resolve_dispute(
            &e,
            dispute_resolver,
            trustless_work_address,
            distributions,
        )?;
        DisputeResolved { escrow }.publish(&e);
        Ok(())
    }

    pub fn dispute_escrow(e: Env, signer: Address, reason: String) -> Result<(), EscrowError> {
        let escrow = DisputeManager::dispute_escrow(&e, signer, reason)?;
        EscrowDisputed { escrow }.publish(&e);
        Ok(())
    }

    pub fn withdraw_remaining_funds(
        e: Env,
        dispute_resolver: Address,
        trustless_work_address: Address,
        distributions: Map<Address, i128>,
    ) -> Result<(), EscrowError> {
        DisputeManager::withdraw_remaining_funds(
            &e,
            dispute_resolver,
            trustless_work_address,
            distributions,
        )?;
        Ok(())
    }
}
