use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, String, Symbol, Val, Vec};

use crate::core::{DisputeManager, EscrowManager, MilestoneManager};
use crate::error::ContractError;
use crate::events::handler::{
    ChgEsc, DisEsc, DisputeResolved, EscrowDisputed, ExtTtlEvt, FundEsc, InitEsc,
    MilestoneApproved, MilestoneStatusChanged,
};
use crate::storage::types::{AddressBalance, Escrow};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn __constructor() {}

    pub fn deploy(
        env: Env,
        deployer: Address,
        wasm_hash: BytesN<32>,
        salt: BytesN<32>,
        init_fn: Symbol,
        init_args: Vec<Val>,
        constructor_args: Vec<Val>,
    ) -> (Address, Val) {
        if deployer != env.current_contract_address() {
            deployer.require_auth();
        }

        let deployed_address = env
            .deployer()
            .with_address(deployer, salt)
            .deploy_v2(wasm_hash, constructor_args);

        let res: Val = env.invoke_contract(&deployed_address, &init_fn, init_args);
        (deployed_address, res)
    }

    ////////////////////////
    // Escrow /////
    ////////////////////////

    pub fn initialize_escrow(e: &Env, escrow_properties: Escrow) -> Result<Escrow, ContractError> {
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
    ) -> Result<(), ContractError> {
        EscrowManager::fund_escrow(e, &signer, &expected_escrow, amount)?;
        FundEsc { signer, amount }.publish(e);
        Ok(())
    }

    pub fn release_funds(
        e: &Env,
        release_signer: Address,
        trustless_work_address: Address,
    ) -> Result<(), ContractError> {
        EscrowManager::release_funds(e, &release_signer, &trustless_work_address)?;
        DisEsc { release_signer }.publish(e);
        Ok(())
    }

    pub fn update_escrow(
        e: &Env,
        plataform_address: Address,
        escrow_properties: Escrow,
    ) -> Result<Escrow, ContractError> {
        let updated_escrow = EscrowManager::change_escrow_properties(
            e,
            &plataform_address,
            escrow_properties.clone(),
        )?;
        ChgEsc {
            platform: plataform_address,
            engagement_id: escrow_properties.engagement_id.clone(),
            new_escrow_properties: updated_escrow.clone(),
        }
        .publish(e);
        Ok(updated_escrow)
    }

    pub fn get_escrow(e: &Env) -> Result<Escrow, ContractError> {
        EscrowManager::get_escrow(e)
    }

    pub fn get_escrow_by_contract_id(
        e: &Env,
        contract_id: Address,
    ) -> Result<Escrow, ContractError> {
        EscrowManager::get_escrow_by_contract_id(e, &contract_id)
    }

    pub fn get_multiple_escrow_balances(
        e: &Env,
        addresses: Vec<Address>,
    ) -> Result<Vec<AddressBalance>, ContractError> {
        EscrowManager::get_multiple_escrow_balances(e, addresses)
    }

    ////////////////////////
    // Admin / TTL /////
    ////////////////////////

    pub fn extend_contract_ttl(
        e: &Env,
        platform_address: Address,
        ledgers_to_extend: u32,
    ) -> Result<(), ContractError> {
        platform_address.require_auth();

        let escrow = EscrowManager::get_escrow(e)?;
        if platform_address != escrow.roles.platform_address {
            return Err(ContractError::OnlyPlatformAddressExecuteThisFunction);
        }

        let min_ledgers = 1u32;
        e.storage()
            .instance()
            .extend_ttl(min_ledgers, ledgers_to_extend);

        ExtTtlEvt {
            platform: platform_address,
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
        milestone_index: i128,
        new_status: String,
        new_evidence: Option<String>,
        service_provider: Address,
    ) -> Result<(), ContractError> {
        let escrow = MilestoneManager::change_milestone_status(
            &e,
            milestone_index,
            new_status,
            new_evidence,
            service_provider,
        )?;
        MilestoneStatusChanged { escrow }.publish(&e);
        Ok(())
    }

    pub fn approve_milestone(
        e: Env,
        milestone_index: i128,
        approver: Address,
    ) -> Result<(), ContractError> {
        let escrow =
            MilestoneManager::change_milestone_approved_flag(&e, milestone_index, approver)?;
        MilestoneApproved { escrow }.publish(&e);
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
    ) -> Result<(), ContractError> {
        let escrow = DisputeManager::resolve_dispute(
            &e,
            dispute_resolver,
            trustless_work_address,
            distributions,
        )?;
        DisputeResolved { escrow }.publish(&e);
        Ok(())
    }

    pub fn dispute_escrow(e: Env, signer: Address) -> Result<(), ContractError> {
        let escrow = DisputeManager::dispute_escrow(&e, signer)?;
        EscrowDisputed { escrow }.publish(&e);
        Ok(())
    }
}
