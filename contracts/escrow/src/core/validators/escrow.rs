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
        return Err(ContractError::EscrowAlreadyReleased);
    }

    if escrow.flags.resolved {
        return Err(ContractError::EscrowAlreadyResolved);
    }

    if release_signer != &escrow.roles.release_signer {
        return Err(ContractError::OnlyReleaseSignerCanReleaseEarnings);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMilestoneDefined);
    }

    if !escrow.milestones.iter().all(|milestone| milestone.approved) {
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
    new_escrow: &Escrow,
    platform_address: &Address,
    contract_balance: i128,
) -> Result<(), ContractError> {
    if existing_escrow.roles.platform_address != new_escrow.roles.platform_address {
        return Err(ContractError::PlatformAddressCannotBeChanged);
    }

    if existing_escrow.flags.disputed {
        return Err(ContractError::EscrowOpenedForDisputeResolution);
    }

    for milestone in existing_escrow.milestones.iter() {
        if milestone.approved {
            return Err(ContractError::MilestoneApprovedCantChangeEscrowProperties);
        }
    }

    if new_escrow.flags.released
        || new_escrow.flags.disputed
        || new_escrow.flags.resolved
        || new_escrow.milestones.iter().any(|m| m.approved)
    {
        return Err(ContractError::FlagsMustBeFalse);
    }

    if platform_address != &existing_escrow.roles.platform_address {
        return Err(ContractError::OnlyPlatformAddressExecuteThisFunction);
    }

    // If the contract has funds, updates must be append-only.
    // If there are no funds, allow modifying properties (with standard validations).
    if contract_balance > 0 {
        // Append-only policy when funds exist: All other escrow properties and the existing milestones must remain identical.
        if existing_escrow.engagement_id != new_escrow.engagement_id
            || existing_escrow.title != new_escrow.title
            || existing_escrow.description != new_escrow.description
            || existing_escrow.roles != new_escrow.roles
            || existing_escrow.amount != new_escrow.amount
            || existing_escrow.platform_fee != new_escrow.platform_fee
            || existing_escrow.flags != new_escrow.flags
            || existing_escrow.trustline != new_escrow.trustline
            || existing_escrow.receiver_memo != new_escrow.receiver_memo
        {
            return Err(ContractError::EscrowPropertiesMismatch);
        }

        let old_len = existing_escrow.milestones.len();
        let new_len = new_escrow.milestones.len();

        if new_len < old_len {
            return Err(ContractError::EscrowPropertiesMismatch);
        }

        for i in 0..old_len {
            if existing_escrow.milestones.get(i).unwrap()
                != new_escrow.milestones.get(i).unwrap()
            {
                return Err(ContractError::EscrowPropertiesMismatch);
            }
        }

        if new_escrow.amount == 0 {
            return Err(ContractError::AmountCannotBeZero);
        }

        if new_len > 50 {
            return Err(ContractError::TooManyMilestones);
        }

        if new_len == 0 {
            return Err(ContractError::NoMilestoneDefined);
        }
    } else {
        if new_escrow.amount == 0 {
            return Err(ContractError::AmountCannotBeZero);
        }

        if new_escrow.milestones.len() > 50 {
            return Err(ContractError::TooManyMilestones);
        }

        if new_escrow.milestones.is_empty() {
            return Err(ContractError::NoMilestoneDefined);
        }
    }

    Ok(())
}

#[inline]
pub fn validate_initialize_escrow_conditions(
    e: &Env,
    escrow_properties: Escrow,
) -> Result<(), ContractError> {
    if e.storage().instance().has(&DataKey::Escrow) {
        return Err(ContractError::EscrowAlreadyInitialized);
    }

    if escrow_properties.flags.released
        || escrow_properties.flags.disputed
        || escrow_properties.flags.resolved
        || escrow_properties.milestones.iter().any(|m| m.approved)
    {
        return Err(ContractError::FlagsMustBeFalse);
    }

    if escrow_properties.milestones.is_empty() {
        return Err(ContractError::NoMilestoneDefined);
    }

    let max_bps_percentage: u32 = 99 * 100;
    if escrow_properties.platform_fee > max_bps_percentage {
        return Err(ContractError::PlatformFeeTooHigh);
    }

    if escrow_properties.amount == 0 {
        return Err(ContractError::AmountCannotBeZero);
    }

    if escrow_properties.milestones.len() > 50 {
        return Err(ContractError::TooManyMilestones);
    }

    Ok(())
}

#[inline]
pub fn validate_fund_escrow_conditions(
    amount: i128,
    stored_escrow: &Escrow,
    expected_escrow: &Escrow,
) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::AmountCannotBeZero);
    }

    if !stored_escrow.eq(&expected_escrow) {
        return Err(ContractError::EscrowPropertiesMismatch);
    }

    Ok(())
}
