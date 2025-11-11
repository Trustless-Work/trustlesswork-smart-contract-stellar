use soroban_sdk::{Address, Env};

use crate::{
    error::ContractError,
    storage::types::{DataKey, Escrow},
};

#[inline]
pub fn validate_release_conditions(
    escrow: &Escrow,
    release_signer: &Address,
) -> Result<(), ContractError> {
    if escrow.flags.released {
        return Err(ContractError::EscrowAlreadyResolved);
    }

    if release_signer != &escrow.roles.release_signer {
        return Err(ContractError::OnlyReleaseSignerCanReleaseEarnings);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMileStoneDefined);
    }

    if !escrow
        .milestones
        .iter()
        .all(|milestone| milestone.approved)
    {
        return Err(ContractError::EscrowNotCompleted);
    }

    if escrow.flags.disputed {
        return Err(ContractError::EscrowOpenedForDisputeResolution);
    }

    Ok(())
}

#[inline]
pub fn validate_escrow_property_change_conditions(
    existing_escrow: &Escrow,
    platform_address: &Address,
    contract_balance: i128,
) -> Result<(), ContractError> {

    if platform_address != &existing_escrow.roles.platform_address {
        return Err(ContractError::OnlyPlatformAddressExecuteThisFunction);
    }

    for milestone in existing_escrow.milestones.iter() {
        if milestone.approved {
            return Err(ContractError::MilestoneApprovedCantChangeEscrowProperties);
        }
    }

    if contract_balance > 0 {
        return Err(ContractError::EscrowHasFunds);
    }

    if existing_escrow.flags.disputed {
        return Err(ContractError::EscrowOpenedForDisputeResolution);
    }

    Ok(())
}

#[inline]
pub fn validate_initialize_escrow_conditions(
    e: Env,
    escrow_properties: Escrow,
) -> Result<(), ContractError> {
    if e.storage().instance().has(&DataKey::Escrow) {
        return Err(ContractError::EscrowAlreadyInitialized);
    }

    if escrow_properties.amount == 0 {
        return Err(ContractError::AmountCannotBeZero);
    }

    if escrow_properties.milestones.len() > 10 {
        return Err(ContractError::TooManyMilestones);
    }

    Ok(())
}
