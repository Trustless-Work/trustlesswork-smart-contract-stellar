use soroban_sdk::{Address, Env, Vec};

use crate::{
    error::ContractError,
    storage::types::{DataKey, Escrow, Milestone},
};

#[inline]
pub fn validate_release_conditions(
    escrow: &Escrow,
    release_signer: &Address,
    milestone: &Milestone,
    milestone_index: u32,
) -> Result<(), ContractError> {
    if milestone.flags.released {
        return Err(ContractError::MilestoneAlreadyReleased);
    }

    if milestone.flags.resolved {
        return Err(ContractError::EscrowAlreadyResolved);
    }

    if release_signer != &escrow.roles.release_signer {
        return Err(ContractError::OnlyReleaseSignerCanReleaseEarnings);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMileStoneDefined);
    }

    if !milestone.flags.approved {
        return Err(ContractError::MilestoneNotCompleted);
    }

    if milestone.flags.disputed {
        return Err(ContractError::MilestoneOpenedForDisputeResolution);
    }

    if milestone_index >= escrow.milestones.len() {
        return Err(ContractError::InvalidMileStoneIndex);
    }

    Ok(())
}

#[inline]
pub fn validate_escrow_property_change_conditions(
    existing_escrow: &Escrow,
    new_escrow: &Escrow,
    platform_address: &Address,
    contract_balance: i128,
    milestones: Vec<Milestone>,
) -> Result<(), ContractError> {
    if existing_escrow.roles.platform_address != new_escrow.roles.platform_address {
        return Err(ContractError::PlatformAddressCannotBeChanged);
    }

    for milestone in milestones.iter() {
        if milestone.flags.disputed
            || milestone.flags.released
            || milestone.flags.resolved
            || milestone.flags.approved
        {
            return Err(ContractError::FlagsMustBeFalse);
        }
        if milestone.flags.disputed {
            return Err(ContractError::MilestoneOpenedForDisputeResolution);
        }
        if milestone.flags.approved {
            return Err(ContractError::MilestoneApprovedCantChangeEscrowProperties);
        }
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
            || existing_escrow.platform_fee != new_escrow.platform_fee
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
            let old_m = existing_escrow.milestones.get(i).unwrap();
            let new_m = new_escrow.milestones.get(i).unwrap();
            if old_m != new_m {
                return Err(ContractError::EscrowPropertiesMismatch);
            }
        }

        for m in new_escrow.milestones.iter() {
            if m.amount == 0 {
                return Err(ContractError::AmountCannotBeZero);
            }
        }

        if new_len > 50 {
            return Err(ContractError::TooManyMilestones);
        }

        if new_len == 0 {
            return Err(ContractError::NoMileStoneDefined);
        }
    } else {
        for m in new_escrow.milestones.iter() {
            if m.amount == 0 {
                return Err(ContractError::AmountCannotBeZero);
            }
        }

        if new_escrow.milestones.len() > 50 {
            return Err(ContractError::TooManyMilestones);
        }

        if new_escrow.milestones.is_empty() {
            return Err(ContractError::NoMileStoneDefined);
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

    for milestone in escrow_properties.milestones.iter() {
        if milestone.flags.disputed
            || milestone.flags.released
            || milestone.flags.resolved
            || milestone.flags.approved
        {
            return Err(ContractError::FlagsMustBeFalse);
        }
        if milestone.amount == 0 {
            return Err(ContractError::AmountCannotBeZero);
        }
    }

    if escrow_properties.milestones.is_empty() {
        return Err(ContractError::NoMileStoneDefined);
    }

    let max_bps_percentage: u32 = 99 * 100;
    if escrow_properties.platform_fee > max_bps_percentage {
        return Err(ContractError::PlatformFeeTooHigh);
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
