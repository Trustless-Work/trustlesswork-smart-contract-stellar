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
pub fn validate_escrow_conditions(
    existing_escrow: Option<&Escrow>,
    new_escrow: &Escrow,
    platform_address: Option<&Address>,
    contract_balance: Option<i128>,
    is_init: bool,
) -> Result<(), ContractError> {
    let max_bps_percentage: u32 = 99 * 100;
    if new_escrow.platform_fee > max_bps_percentage {
        return Err(ContractError::PlatformFeeTooHigh);
    }

    const TRUSTLESS_WORK_FEE_BPS: u32 = 30;
    if (new_escrow.platform_fee as u32) + TRUSTLESS_WORK_FEE_BPS > 10_000 {
        return Err(ContractError::PlatformFeeTooHigh);
    }

    if new_escrow.amount < 0 {
        return Err(ContractError::AmountCannotBeZero);
    }

    if new_escrow.milestones.is_empty() {
        return Err(ContractError::NoMilestoneDefined);
    }
    if new_escrow.milestones.len() > 50 {
        return Err(ContractError::TooManyMilestones);
    }

    if is_init {
        if new_escrow.flags.released
            || new_escrow.flags.disputed
            || new_escrow.flags.resolved
            || new_escrow.milestones.iter().any(|m| m.approved)
        {
            return Err(ContractError::FlagsMustBeFalse);
        }
    } else {
        let existing = existing_escrow.ok_or(ContractError::EscrowNotFound)?;

        let caller = platform_address.ok_or(ContractError::OnlyPlatformAddressExecuteThisFunction)?;
        if caller != &existing.roles.platform_address {
            return Err(ContractError::OnlyPlatformAddressExecuteThisFunction);
        }

        if existing.roles.platform_address != new_escrow.roles.platform_address {
            return Err(ContractError::PlatformAddressCannotBeChanged);
        }

        if existing.flags.disputed {
            return Err(ContractError::EscrowOpenedForDisputeResolution);
        }

        if new_escrow.flags.released
            || new_escrow.flags.disputed
            || new_escrow.flags.resolved
        {
            return Err(ContractError::FlagsMustBeFalse);
        }

        let has_funds = contract_balance.unwrap_or(0) > 0;
        if has_funds {
            if existing.engagement_id != new_escrow.engagement_id
                || existing.title != new_escrow.title
                || existing.description != new_escrow.description
                || existing.roles != new_escrow.roles
                || existing.amount != new_escrow.amount
                || existing.platform_fee != new_escrow.platform_fee
                || existing.flags != new_escrow.flags
                || existing.trustline != new_escrow.trustline
                || existing.receiver_memo != new_escrow.receiver_memo
            {
                return Err(ContractError::EscrowPropertiesMismatch);
            }

            let old_len = existing.milestones.len();
            let new_len = new_escrow.milestones.len();
            if new_len < old_len {
                return Err(ContractError::EscrowPropertiesMismatch);
            }
            for i in 0..old_len {
                if existing.milestones.get(i).unwrap() != new_escrow.milestones.get(i).unwrap() {
                    return Err(ContractError::EscrowPropertiesMismatch);
                }
            }

            for i in old_len..new_len {
                if new_escrow.milestones.get(i).unwrap().approved {
                    return Err(ContractError::FlagsMustBeFalse);
                }
            }
        } else {
            if existing.milestones.iter().any(|m| m.approved) {
                return Err(ContractError::MilestoneApprovedCantChangeEscrowProperties);
            }

            if new_escrow.milestones.iter().any(|m| m.approved) {
                return Err(ContractError::FlagsMustBeFalse);
            }
        }
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
    validate_escrow_conditions(
        Some(existing_escrow),
        new_escrow,
        Some(platform_address),
        Some(contract_balance),
        false,
    )
}

#[inline]
pub fn validate_initialize_escrow_conditions(
    e: &Env,
    escrow_properties: Escrow,
) -> Result<(), ContractError> {
    if e.storage().instance().has(&DataKey::Escrow) {
        return Err(ContractError::EscrowAlreadyInitialized);
    }
    validate_escrow_conditions(None, &escrow_properties, None, None, true)
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
