use crate::error::EscrowError;
use crate::storage::{DataKey, EscrowData, ReleaseData, RELEASE_COUNT_LIMIT};
use soroban_sdk::{address::Address, log, AddressString, Vec};

/// Check reentrancy guard
fn check_reentrancy(contract: &crate::EscrowContract, caller: &Address) -> Result<(), EscrowError> {
    if contract.storage().persistent().has(&DataKey::Reentrancy) {
        return Err(EscrowError::Reentrancy);
    }
    Ok(())
}

/// Set reentrancy guard
fn set_reentrancy_guard(contract: &crate::EscrowContract) {
    contract
        .storage()
        .persistent()
        .set(&DataKey::Reentrancy, &());
}

/// Clear reentrancy guard
fn clear_reentrancy_guard(contract: &crate::EscrowContract) {
    contract
        .storage()
        .persistent()
        .remove(&DataKey::Reentrancy);
}

/// Get dispute ID from storage
fn get_dispute_id(contract: &crate::EscrowContract) -> Result<u64, EscrowError> {
    match contract.storage().persistent().get(&DataKey::DisputeId) {
        Some(id) => Ok(id),
        None => Err(EscrowError::DisputeNotInProgress),
    }
}

/// Set dispute ID in storage
fn set_dispute_id(contract: &crate::EscrowContract, id: u64) {
    contract
        .storage()
        .persistent()
        .set(&DataKey::DisputeId, &id);
}

/// Clear dispute ID from storage
fn clear_dispute_id(contract: &crate::EscrowContract) {
    contract
        .storage()
        .persistent()
        .remove(&DataKey::DisputeId);
}

/// Initialize a dispute for an escrow
pub fn init_dispute(
    contract: &crate::EscrowContract,
    escrow_id: u64,
    claimant: Address,
    arbitrator: Address,
) -> Result<(), EscrowError> {
    check_reentrancy(contract, &claimant)?;

    // Verify the escrow exists and is active
    let escrow_data: EscrowData = contract.get_escrow_data(escrow_id)?;

    if escrow_data.status != crate::types::EscrowStatus::Active {
        return Err(EscrowError::NotInitiated);
    }

    // Verify the claimant is the receiver of the escrow
    if escrow_data.receiver != claimant.clone() {
        return Err(EscrowError::InvalidClaim);
    }

    // Check if there's already a dispute in progress
    if contract
        .storage()
        .persistent()
        .has(&DataKey::DisputeId)
    {
        return Err(EscrowError::AlreadyDisputed);
    }

    // Set up dispute state
    let dispute_id = contract.next_dispute_id()?;
    set_dispute_id(contract, dispute_id);

    // Store dispute data
    contract
        .storage()
        .persistent()
        .set(
            &DataKey::Dispute(dispute_id),
            &crate::types::DisputeData {
                escrow_id,
                claimant: claimant.clone(),
                arbitrator: arbitrator.clone(),
                initiated_at: contract.env().current_ledger_timestamp(),
                resolved: false,
                outcome: None,
            },
        );

    // Update escrow status
    let mut updated_escrow = escrow_data;
    updated_escrow.status = crate::types::EscrowStatus::Disputed;
    contract.set_escrow_data(escrow_id, &updated_escrow)?;

    // Set reentrancy guard
    set_reentrancy_guard(contract);

    log!(
        contract,
        "Dispute initialized: escrow_id={}, dispute_id={}, claimant={}, arbitrator={}",
        escrow_id,
        dispute_id,
        AddressString::new(&claimant),
        AddressString::new(&arbitrator)
    );

    clear_reentrancy_guard(contract);

    Ok(())
}

/// Resolve a dispute with an outcome
pub fn resolve_dispute(
    contract: &crate::EscrowContract,
    dispute_id: u64,
    outcome: crate::types::DisputeOutcome,
) -> Result<(), EscrowError> {
    check_reentrancy(contract, &outcome.recipient())?;

    // Verify the dispute exists and is in progress
    let dispute_data = contract.get_dispute_data(dispute_id)?;

    if dispute_data.resolved {
        return Err(EscrowError::AlreadyResolved);
    }

    // Verify the caller is the arbitrator
    let caller = Address::get_current_contract_address();
    // Note: In Soroban, we'd typically check the actual signer. This is a simplified version.
    // The actual implementation would need to verify the message signer.

    // Resolve the dispute
    let mut updated_dispute = dispute_data.clone();
    updated_dispute.resolved = true;
    updated_dispute.outcome = Some(outcome.clone());
    contract.set_dispute_data(dispute_id, &updated_dispute)?;

    // Execute the outcome
    match &outcome {
        crate::types::DisputeOutcome::ReleaseToReceiver { amount } => {
            contract.execute_release(
                dispute_data.escrow_id,
                dispute_data.claimant.clone(),
                *amount,
            )?;
        }
        crate::types::DisputeOutcome::ReleaseToFunder { amount } => {
            contract.execute_refund(
                dispute_data.escrow_id,
                dispute_data.claimant.clone(),
                *amount,
            )?;
        }
        crate::types::DisputeOutcome::PartialRelease {
            receiver_amount,
            funder_amount,
        } => {
            contract.execute_release(
                dispute_data.escrow_id,
                dispute_data.claimant.clone(),
                *receiver_amount,
            )?;
            contract.execute_refund(
                dispute_data.escrow_id,
                dispute_data.claimant.clone(),
                *funder_amount,
            )?;
        }
    }

    // Clear dispute state
    clear_dispute_id(contract);

    log!(
        contract,
        "Dispute resolved: dispute_id={}, outcome={:?}",
        dispute_id,
        outcome
    );

    Ok(())
}

/// Cancel a dispute
pub fn cancel_dispute(contract: &crate::EscrowContract) -> Result<(), EscrowError> {
    check_reentrancy(contract, &Address::get_current_contract_address())?;

    // Get the current dispute
    let dispute_id = get_dispute_id(contract)?;
    let dispute_data = contract.get_dispute_data(dispute_id)?;

    if dispute_data.resolved {
        return Err(EscrowError::AlreadyResolved);
    }

    // Restore escrow status
    let mut escrow_data = contract.get_escrow_data(dispute_data.escrow_id)?;
    escrow_data.status = crate::types::EscrowStatus::Active;
    contract.set_escrow_data(dispute_data.escrow_id, &escrow_data)?;

    // Clear dispute data
    contract
        .storage()
        .persistent()
        .remove(&DataKey::Dispute(dispute_id));

    // Clear dispute ID
    clear_dispute_id(contract);

    log!(
        contract,
        "Dispute cancelled: dispute_id={}",
        dispute_id
    );

    Ok(())
}

/// Get dispute info
pub fn get_dispute_info(
    contract: &crate::EscrowContract,
    dispute_id: u64,
) -> Result<crate::types::DisputeInfo, EscrowError> {
    let dispute_data = contract.get_dispute_data(dispute_id)?;

    Ok(crate::types::DisputeInfo {
        escrow_id: dispute_data.escrow_id,
        claimant: dispute_data.claimant,
        arbitrator: dispute_data.arbitrator,
        initiated_at: dispute_data.initiated_at,
        resolved: dispute_data.resolved,
        outcome: dispute_data.outcome,
    })
}

/// Get the next dispute ID
pub fn get_next_dispute_id(contract: &crate::EscrowContract) -> u64 {
    match contract.storage().persistent().get(&DataKey::NextDisputeId) {
        Some(id) => id + 1,
        None => 1,
    }
}
