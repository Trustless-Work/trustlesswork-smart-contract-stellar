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
fn validate_escrow_conditions(
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
    if new_escrow.milestones.is_empty() {
        return Err(ContractError::NoMileStoneDefined);
    }
    if new_escrow.milestones.len() > 50 {
        return Err(ContractError::TooManyMilestones);
    }

    for m in new_escrow.milestones.iter() {
        if m.amount <= 0 {
            return Err(ContractError::AmountCannotBeZero);
        }
    }

    if is_init {
        for m in new_escrow.milestones.iter() {
            if m.flags.disputed || m.flags.released || m.flags.resolved || m.flags.approved {
                return Err(ContractError::FlagsMustBeFalse);
            }
        }
    } else {
        let existing = existing_escrow.ok_or(ContractError::EscrowNotFound)?;
        let caller =
            platform_address.ok_or(ContractError::OnlyPlatformAddressExecuteThisFunction)?;
        if caller != &existing.roles.platform_address {
            return Err(ContractError::OnlyPlatformAddressExecuteThisFunction);
        }
        if existing.roles.platform_address != new_escrow.roles.platform_address {
            return Err(ContractError::PlatformAddressCannotBeChanged);
        }

        let has_funds = contract_balance.unwrap_or(0) > 0;

        let old_len = existing.milestones.len();
        let new_len = new_escrow.milestones.len();
        let overlap = if new_len < old_len { new_len } else { old_len };
        for i in 0..overlap {
            let old_m = existing.milestones.get(i).unwrap();
            let new_m = new_escrow.milestones.get(i).unwrap();
            if old_m.flags != new_m.flags {
                return Err(ContractError::EscrowPropertiesMismatch);
            }
        }
        if new_len > old_len {
            for i in old_len..new_len {
                let m = new_escrow.milestones.get(i).unwrap();
                if m.flags.disputed || m.flags.released || m.flags.resolved || m.flags.approved {
                    return Err(ContractError::FlagsMustBeFalse);
                }
            }
        }

        if has_funds {
            if existing.engagement_id != new_escrow.engagement_id
                || existing.title != new_escrow.title
                || existing.description != new_escrow.description
                || existing.roles != new_escrow.roles
                || existing.platform_fee != new_escrow.platform_fee
                || existing.trustline != new_escrow.trustline
                || existing.receiver_memo != new_escrow.receiver_memo
            {
                return Err(ContractError::EscrowPropertiesMismatch);
            }
            if new_len < old_len {
                return Err(ContractError::EscrowPropertiesMismatch);
            }
            for i in 0..old_len {
                let old_m = existing.milestones.get(i).unwrap();
                let new_m = new_escrow.milestones.get(i).unwrap();
                if old_m != new_m {
                    return Err(ContractError::EscrowPropertiesMismatch);
                }
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
    _milestones: Vec<Milestone>,
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
    escrow_balance: i128,
) -> Result<(), ContractError> {
    if e.storage().persistent().has(&DataKey::Escrow) {
        return Err(ContractError::EscrowAlreadyInitialized);
    }

    if escrow_balance > 0 {
        return Err(ContractError::EscrowBalanceMustBeZeroOnInitialization);
    }

    validate_escrow_conditions(None, &escrow_properties, None, None, true)
}

#[inline]
pub fn validate_fund_escrow_conditions(
    amount: i128,
    balance: i128,
    stored_escrow: &Escrow,
    expected_escrow: &Escrow,
) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::AmountCannotBeZero);
    }

    if !stored_escrow.eq(&expected_escrow) {
        return Err(ContractError::EscrowPropertiesMismatch);
    }

    if balance < amount {
        return Err(ContractError::InsufficientFundsForEscrowFunding);
    }

    Ok(())
}
