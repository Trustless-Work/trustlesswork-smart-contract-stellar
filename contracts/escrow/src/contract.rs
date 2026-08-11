use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, String, Symbol, Val, Vec};

use crate::core::{DisputeManager, EscrowManager, MilestoneManager};
use crate::error::{CctpError, EscrowError, MilestoneError};
use crate::events::handler::{
    DisputeResolved, EscrowDisputed, EscrowUpdated, FundEsc, FundsWithdrawn, InitEsc,
    MilestoneStatusChanged, MilestonesApproved, MilestonesManaged, ReleaseEsc, TtlExtended,
};
use crate::storage::types::{
    AddressBalance, CrossChainDestination, DataKey, DistributionEntry, Escrow, Milestone,
    MilestoneStatusEntry, MilestoneStatusUpdate, MilestoneUpdate,
};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn __constructor(e: Env, admin: Address, approved_wasm_hash: BytesN<32>) {
        e.storage().persistent().set(&DataKey::Admin, &admin);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::Admin, 17280, 31536000);
        e.storage()
            .persistent()
            .set(&DataKey::ApprovedWasmHash, &approved_wasm_hash);
        e.storage()
            .persistent()
            .extend_ttl(&DataKey::ApprovedWasmHash, 17280, 31536000);
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

        let approved_wasm_hash: BytesN<32> = env
            .storage()
            .persistent()
            .get(&DataKey::ApprovedWasmHash)
            .ok_or(EscrowError::OnlyAdminAddressExecuteThisFunction)?;

        if wasm_hash != approved_wasm_hash {
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
            engagement_id: initialized_escrow.engagement_id.clone(),
            amount: initialized_escrow.amount,
            platform_fee: initialized_escrow.platform_fee,
            trustline: initialized_escrow.trustline.address.clone(),
            destination_domain: initialized_escrow.receiver.cctp.destination_domain,
            mint_recipient: initialized_escrow.receiver.cctp.mint_recipient.clone(),
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
        let engagement_id = expected_escrow.engagement_id.clone();
        let funded_total = EscrowManager::fund_escrow(e, &signer, &expected_escrow, amount)?;
        FundEsc {
            engagement_id,
            funder: signer,
            amount,
            funded_total,
        }
        .publish(e);
        Ok(())
    }

    pub fn release_funds(
        e: &Env,
        release_signer: Address,
        trustless_work_address: Address,
    ) -> Result<(), EscrowError> {
        let (escrow, fee_result) =
            EscrowManager::release_funds(e, &release_signer, &trustless_work_address)?;
        ReleaseEsc {
            engagement_id: escrow.engagement_id,
            release_signer,
            destination_domain: escrow.receiver.cctp.destination_domain,
            mint_recipient: escrow.receiver.cctp.mint_recipient.clone(),
            amount: escrow.amount,
            platform_fee: fee_result.platform_fee,
            trustless_work_fee: fee_result.trustless_work_fee,
            net_amount: fee_result.receiver_amount,
        }
        .publish(e);
        Ok(())
    }

    pub fn update_escrow(
        e: &Env,
        admin_address: Address,
        escrow_properties: Escrow,
    ) -> Result<Escrow, EscrowError> {
        let updated_escrow =
            EscrowManager::change_escrow_properties(e, &admin_address, escrow_properties)?;
        EscrowUpdated {
            engagement_id: updated_escrow.engagement_id.clone(),
            admin: admin_address,
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
        let added_count = new_milestones.len();
        let updated_count = milestone_updates.len();
        let updated_escrow =
            EscrowManager::manage_milestones(e, &admin_address, new_milestones, milestone_updates)?;
        MilestonesManaged {
            engagement_id: updated_escrow.engagement_id.clone(),
            admin: admin_address,
            added_count,
            updated_count,
        }
        .publish(e);
        Ok(updated_escrow)
    }

    pub fn get_escrow(e: &Env) -> Result<Escrow, EscrowError> {
        EscrowManager::get_escrow(e)
    }

    ////////////////////////
    // Cross-chain payout (receiver-controlled) /////
    ////////////////////////

    pub fn set_cross_chain_destination(
        e: &Env,
        receiver: Address,
        destination_domain: u32,
        mint_recipient: BytesN<32>,
        max_fee: i128,
    ) -> Result<(), CctpError> {
        EscrowManager::set_cross_chain_destination(
            e,
            &receiver,
            destination_domain,
            &mint_recipient,
            max_fee,
        )
    }

    pub fn get_cross_chain_destination(e: &Env) -> Result<CrossChainDestination, CctpError> {
        EscrowManager::get_cross_chain_destination(e)
    }

    pub fn get_escrow_by_contract_id(e: &Env, contract_id: Address) -> Result<Escrow, EscrowError> {
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

        TtlExtended {
            engagement_id: escrow.engagement_id,
            admin,
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
        let mut status_entries: Vec<MilestoneStatusEntry> = Vec::new(&e);
        for update in updates.iter() {
            status_entries.push_back(MilestoneStatusEntry {
                index: update.milestone_index,
                status: update.new_status.clone(),
            });
        }
        let escrow =
            MilestoneManager::change_milestone_status(&e, updates, service_provider.clone())?;
        MilestoneStatusChanged {
            engagement_id: escrow.engagement_id,
            service_provider,
            updates: status_entries,
        }
        .publish(&e);
        Ok(())
    }

    pub fn approve_milestones(
        e: Env,
        milestone_indices: Vec<u32>,
        approver: Address,
    ) -> Result<(), MilestoneError> {
        let escrow =
            MilestoneManager::approve_milestones(&e, milestone_indices.clone(), approver.clone())?;
        MilestonesApproved {
            engagement_id: escrow.engagement_id,
            approver,
            milestone_indices,
        }
        .publish(&e);
        Ok(())
    }

    pub fn approve_and_release_milestones(
        e: Env,
        signer: Address,
        trustless_work_address: Address,
        milestone_indices: Vec<u32>,
    ) -> Result<(), EscrowError> {
        let escrow = EscrowManager::get_escrow(&e)?;
        if !escrow.roles.approvers.contains(&signer)
            || !escrow.roles.release_signers.contains(&signer)
        {
            return Err(EscrowError::SignerMustBeApproverAndReleaseSigner);
        }
        signer.require_auth();
        let updated_escrow = MilestoneManager::approve_milestones_inner(
            &e,
            milestone_indices.clone(),
            signer.clone(),
        )?;
        MilestonesApproved {
            engagement_id: updated_escrow.engagement_id.clone(),
            approver: signer.clone(),
            milestone_indices,
        }
        .publish(&e);
        let (release_escrow, fee_result) =
            EscrowManager::release_funds_inner(&e, &signer, &trustless_work_address)?;
        ReleaseEsc {
            engagement_id: release_escrow.engagement_id,
            release_signer: signer,
            destination_domain: release_escrow.receiver.cctp.destination_domain,
            mint_recipient: release_escrow.receiver.cctp.mint_recipient.clone(),
            amount: release_escrow.amount,
            platform_fee: fee_result.platform_fee,
            trustless_work_fee: fee_result.trustless_work_fee,
            net_amount: fee_result.receiver_amount,
        }
        .publish(&e);
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
        let (escrow, fee_result, net_dists) = DisputeManager::resolve_dispute(
            &e,
            dispute_resolver.clone(),
            trustless_work_address,
            distributions,
        )?;
        let mut dist_entries: Vec<DistributionEntry> = Vec::new(&e);
        for (address, amount) in net_dists.iter() {
            dist_entries.push_back(DistributionEntry { address, amount });
        }
        DisputeResolved {
            engagement_id: escrow.engagement_id,
            dispute_resolver,
            platform_fee: fee_result.platform_fee,
            trustless_work_fee: fee_result.trustless_work_fee,
            distributions: dist_entries,
        }
        .publish(&e);
        Ok(())
    }

    pub fn dispute_escrow(e: Env, signer: Address, reason: String) -> Result<(), EscrowError> {
        let escrow = DisputeManager::dispute_escrow(&e, signer.clone(), reason.clone())?;
        EscrowDisputed {
            engagement_id: escrow.engagement_id,
            signer,
            reason,
        }
        .publish(&e);
        Ok(())
    }

    pub fn withdraw_remaining_funds(
        e: Env,
        dispute_resolver: Address,
        trustless_work_address: Address,
        distributions: Map<Address, i128>,
    ) -> Result<(), EscrowError> {
        let (escrow, fee_result, net_dists) = DisputeManager::withdraw_remaining_funds(
            &e,
            dispute_resolver.clone(),
            trustless_work_address,
            distributions,
        )?;
        let mut dist_entries: Vec<DistributionEntry> = Vec::new(&e);
        for (address, amount) in net_dists.iter() {
            dist_entries.push_back(DistributionEntry { address, amount });
        }
        FundsWithdrawn {
            engagement_id: escrow.engagement_id,
            dispute_resolver,
            platform_fee: fee_result.platform_fee,
            trustless_work_fee: fee_result.trustless_work_fee,
            distributions: dist_entries,
        }
        .publish(&e);
        Ok(())
    }
}
