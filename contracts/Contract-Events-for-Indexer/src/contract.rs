use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, BytesN, Env, String, Symbol, Val, Vec,
};

use crate::core::{DisputeManager, EscrowManager, MilestoneManager};
use crate::error::ContractError;
use crate::storage::types::{AddressBalance, DataKey, Escrow};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

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

    pub fn initialize_escrow(e: Env, escrow_properties: Escrow) -> Result<Escrow, ContractError> {
        let initialized_escrow =
            EscrowManager::initialize_escrow(e.clone(), escrow_properties)?;
        let escrow = initialized_escrow.clone();
        e.events().publish(
            (symbol_short!("escrow"), symbol_short!("created")), 
            (escrow.engagement_id, escrow.amount, escrow.roles.receiver)
        );
        Ok(initialized_escrow)
    }

    pub fn fund_escrow(
        e: Env,
        signer: Address,
        amount_to_deposit: i128,
    ) -> Result<(), ContractError> {
        EscrowManager::fund_escrow(e.clone(), signer.clone(), amount_to_deposit)?;
        let escrow = EscrowManager::get_escrow(e.clone())?;
        e.events().publish(
            (symbol_short!("escrow"), symbol_short!("funded")),
            (e.current_contract_address(), escrow.engagement_id, signer, amount_to_deposit)
        );
        Ok(())
    }

    pub fn release_funds(
        e: Env,
        release_signer: Address,
        trustless_work_address: Address,
    ) -> Result<(), ContractError> {
        EscrowManager::release_funds(
            e.clone(),
            release_signer.clone(),
            trustless_work_address.clone(),
        )?;
        let escrow = EscrowManager::get_escrow(e.clone())?;
        e.events().publish(
            (symbol_short!("escrow"), symbol_short!("released")),
            (escrow.engagement_id, trustless_work_address)
        );
        Ok(())
    }

    pub fn update_escrow(
        e: Env,
        plataform_address: Address,
        escrow_properties: Escrow,
    ) -> Result<Escrow, ContractError> {
        let updated_escrow = EscrowManager::change_escrow_properties(
            e.clone(),
            plataform_address.clone(),
            escrow_properties.clone(),
        )?;
        
        Ok(updated_escrow)
    }

    pub fn get_escrow(e: Env) -> Result<Escrow, ContractError> {
        EscrowManager::get_escrow(e)
    }

    pub fn get_escrow_by_contract_id(
        e: Env,
        contract_id: Address,
    ) -> Result<Escrow, ContractError> {
        EscrowManager::get_escrow_by_contract_id(e, &contract_id)
    }

    pub fn get_multiple_escrow_balances(
        e: Env,
        signer: Address,
        addresses: Vec<Address>,
    ) -> Result<Vec<AddressBalance>, ContractError> {
        EscrowManager::get_multiple_escrow_balances(e, signer, addresses)
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
        MilestoneManager::change_milestone_status(
            e.clone(),
            milestone_index,
            new_status.clone(),
            new_evidence,
            service_provider,
        )?;
        let escrow = EscrowManager::get_escrow(e.clone())?;
        e.events().publish(
            (symbol_short!("escrow"), symbol_short!("marked")),
            (milestone_index, new_status, escrow.engagement_id)
        );
        Ok(())
    }

    pub fn approve_milestone(
        e: Env,
        milestone_index: i128,
        new_flag: bool,
        approver: Address,
    ) -> Result<(), ContractError> {
        MilestoneManager::change_milestone_approved_flag(e.clone(), milestone_index, new_flag, approver)?;
        let escrow = EscrowManager::get_escrow(e.clone())?;
        e.events().publish(
            (symbol_short!("escrow"), symbol_short!("approved")), 
            (milestone_index, new_flag, escrow.engagement_id));
        Ok(())
    }

    ////////////////////////
    // Disputes /////
    ////////////////////////

    pub fn resolve_dispute(
        e: Env,
        dispute_resolver: Address,
        approver_funds: i128,
        receiver_funds: i128,
        trustless_work_address: Address,
    ) -> Result<(), ContractError> {
        DisputeManager::resolve_dispute(
            e.clone(),
            dispute_resolver,
            approver_funds,
            receiver_funds,
            trustless_work_address,
        )?;
        let escrow = EscrowManager::get_escrow(e.clone())?;
        e.events().publish(
            (symbol_short!("escrow"), symbol_short!("resolved")),
            (approver_funds, receiver_funds, escrow.engagement_id)
        );
        Ok(())
    }

    pub fn dispute_escrow(e: Env, signer: Address) -> Result<(), ContractError> {
        DisputeManager::dispute_escrow(e.clone(), signer.clone())?;
        let escrow = EscrowManager::get_escrow(e.clone())?;
        e.events().publish(
            (symbol_short!("escrow"), symbol_short!("raised")), 
            (signer, escrow.engagement_id)
        );
        Ok(())
    }
}
