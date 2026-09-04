use soroban_sdk::{
    contract, contractimpl, Address, BytesN, ContractExecutable, Env, Map, String, Symbol, Val, Vec,
};

use crate::core::{DisputeManager, EscrowManager, MilestoneManager};
use crate::error::{EscrowError, MilestoneError};
use crate::events::handler::{
    hash_string, DisputeResolved, EscrowDisputed, EscrowUpdated, FundEsc, FundsWithdrawn, InitEsc,
    MilestoneStatusChanged, MilestonesApproved, MilestonesManaged, ReleaseEsc, TtlExtended,
};
use crate::storage::types::{
    AddressBalance, DataKey, DistributionEntry, Escrow, EscrowPropertyChanges, Milestone,
    MilestoneAddedEntry, MilestoneStatusEntry, MilestoneStatusUpdate, MilestoneUpdate,
    MilestoneUpdatedEntry,
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
            .deploy_contract(ContractExecutable::Wasm(wasm_hash), constructor_args);

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
            receiver: initialized_escrow.roles.receiver.clone(),
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
            receiver: escrow.roles.receiver,
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
        // Snapshot pre-update state so the event can report what changed.
        let previous = EscrowManager::get_escrow(e)?;
        let updated_escrow =
            EscrowManager::change_escrow_properties(e, &admin_address, escrow_properties)?;
        let changes = EscrowPropertyChanges {
            engagement_id: previous.engagement_id != updated_escrow.engagement_id,
            title: previous.title != updated_escrow.title,
            description: previous.description != updated_escrow.description,
            amount: previous.amount != updated_escrow.amount,
            platform_fee: previous.platform_fee != updated_escrow.platform_fee,
            roles: previous.roles != updated_escrow.roles,
            trustline: previous.trustline != updated_escrow.trustline,
            receiver_memo: previous.receiver_memo != updated_escrow.receiver_memo,
            old_amount: previous.amount,
            new_amount: updated_escrow.amount,
            old_platform_fee: previous.platform_fee,
            new_platform_fee: updated_escrow.platform_fee,
        };
        EscrowUpdated {
            engagement_id: updated_escrow.engagement_id.clone(),
            admin: admin_address,
            changes,
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

        // Hash only after the manager call succeeds, so oversized strings
        // return `StringTooLong` instead of trapping in `hash_string`.
        let added_input = new_milestones.clone();
        let updated_input = milestone_updates.clone();

        let updated_escrow =
            EscrowManager::manage_milestones(e, &admin_address, new_milestones, milestone_updates)?;

        // Per-edit detail, so the event says what changed, not just how many.
        let mut updated_entries: Vec<MilestoneUpdatedEntry> = Vec::new(e);
        for update in updated_input.iter() {
            updated_entries.push_back(MilestoneUpdatedEntry {
                index: update.index,
                new_description_hash: update.new_description.map(|d| hash_string(e, &d)),
            });
        }

        // Appended milestones occupy the final `added_count` slots.
        let base_index = updated_escrow.milestones.len() - added_count;
        let mut added_entries: Vec<MilestoneAddedEntry> = Vec::new(e);
        for (offset, milestone) in added_input.iter().enumerate() {
            added_entries.push_back(MilestoneAddedEntry {
                index: base_index + offset as u32,
                description_hash: hash_string(e, &milestone.description),
            });
        }

        MilestonesManaged {
            engagement_id: updated_escrow.engagement_id.clone(),
            admin: admin_address,
            added_count,
            updated_count,
            added: added_entries,
            updated: updated_entries,
        }
        .publish(e);
        Ok(updated_escrow)
    }

    pub fn get_escrow(e: &Env) -> Result<Escrow, EscrowError> {
        EscrowManager::get_escrow(e)
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

        // FundedAmount gates the post-funding property locks but is only
        // TTL-extended by fund_escrow, so it can archive before Escrow and
        // block the admin paths that read it until it is restored.
        if e.storage().persistent().has(&DataKey::FundedAmount) {
            e.storage().persistent().extend_ttl(
                &DataKey::FundedAmount,
                min_ledgers,
                ledgers_to_extend,
            );
        }

        e.storage()
            .instance()
            .extend_ttl(min_ledgers, ledgers_to_extend);

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
        // Hash only after the manager call succeeds, so oversized evidence
        // returns `StringTooLong` instead of trapping in `hash_string`.
        let described = updates.clone();
        let escrow =
            MilestoneManager::change_milestone_status(&e, updates, service_provider.clone())?;

        let mut status_entries: Vec<MilestoneStatusEntry> = Vec::new(&e);
        for update in described.iter() {
            status_entries.push_back(MilestoneStatusEntry {
                index: update.milestone_index,
                status: update.new_status.clone(),
                // `None` when this update did not touch the evidence.
                evidence_hash: update.new_evidence.map(|ev| hash_string(&e, &ev)),
            });
        }
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
            receiver: release_escrow.roles.receiver,
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
