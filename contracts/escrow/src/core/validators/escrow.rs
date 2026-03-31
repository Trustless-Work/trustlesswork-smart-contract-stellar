use soroban_sdk::{Address, Env, Vec};

use crate::{
    error::EscrowError,
    storage::types::{DataKey, Escrow, Milestone, Roles},
};

#[inline]
fn is_milestone_approved(milestone: &Milestone) -> bool {
    milestone.approvals.quorum > 0
        && milestone.approvals.approval_count >= milestone.approvals.quorum
}

#[inline]
pub fn validate_release_conditions(
    escrow: &Escrow,
    release_signer: &Address,
) -> Result<(), EscrowError> {
    if escrow.flags.released {
        return Err(EscrowError::EscrowAlreadyReleased);
    }

    if escrow.flags.resolved {
        return Err(EscrowError::EscrowAlreadyResolved);
    }

    if !escrow.roles.release_signers.contains(release_signer) {
        return Err(EscrowError::OnlyReleaseSignerCanReleaseEarnings);
    }

    if escrow.milestones.is_empty() {
        return Err(EscrowError::NoMilestoneDefined);
    }

    if !escrow.milestones.iter().all(|m| is_milestone_approved(&m)) {
        return Err(EscrowError::EscrowNotCompleted);
    }

    if escrow.flags.disputed {
        return Err(EscrowError::EscrowOpenedForDisputeResolution);
    }

    Ok(())
}

#[inline]
fn validate_admin_role_overlap(roles: &Roles) -> Result<(), EscrowError> {
    if roles.approvers.contains(&roles.admin)
        || roles.service_providers.contains(&roles.admin)
        || roles.release_signers.contains(&roles.admin)
        || roles.dispute_resolvers.contains(&roles.admin)
        || roles.admin == roles.receiver
    {
        return Err(EscrowError::AdminAddressOverlapsWithOtherRole);
    }
    Ok(())
}

#[inline]
pub fn validate_escrow_conditions(
    existing_escrow: Option<&Escrow>,
    new_escrow: &Escrow,
    admin: Option<&Address>,
    contract_balance: Option<i128>,
    is_init: bool,
) -> Result<(), EscrowError> {
    let max_bps_percentage: u32 = 99 * 100;
    if new_escrow.platform_fee > max_bps_percentage {
        return Err(EscrowError::PlatformFeeTooHigh);
    }
    const TRUSTLESS_WORK_FEE_BPS: u32 = 30;
    if (new_escrow.platform_fee as u32) + TRUSTLESS_WORK_FEE_BPS > 10_000 {
        return Err(EscrowError::PlatformFeeTooHigh);
    }
    if new_escrow.roles.approvers.is_empty() {
        return Err(EscrowError::ApproversListEmpty);
    }
    if new_escrow.roles.service_providers.is_empty() {
        return Err(EscrowError::ServiceProvidersListEmpty);
    }
    if new_escrow.roles.release_signers.is_empty() {
        return Err(EscrowError::ReleaseSignersListEmpty);
    }
    if new_escrow.roles.dispute_resolvers.is_empty() {
        return Err(EscrowError::DisputeResolversListEmpty);
    }
    if new_escrow.amount <= 0 {
        return Err(EscrowError::AmountCannotBeZero);
    }

    if is_init {
        if new_escrow.milestones.is_empty() {
            return Err(EscrowError::NoMilestoneDefined);
        }
        if new_escrow.milestones.len() > 50 {
            return Err(EscrowError::TooManyMilestones);
        }
        if new_escrow.flags.released
            || new_escrow.flags.disputed
            || new_escrow.flags.resolved
            || new_escrow.milestones.iter().any(|m| m.approvals.approval_count > 0)
        {
            return Err(EscrowError::FlagsMustBeFalse);
        }
        if new_escrow.milestones.iter().any(|m| m.approvals.quorum == 0) {
            return Err(EscrowError::QuorumCannotBeZero);
        }
        validate_admin_role_overlap(&new_escrow.roles)?;
    } else {
        let existing = existing_escrow.ok_or(EscrowError::EscrowNotFound)?;
        let caller =
            admin.ok_or(EscrowError::OnlyAdminAddressExecuteThisFunction)?;
        if caller != &existing.roles.admin {
            return Err(EscrowError::OnlyAdminAddressExecuteThisFunction);
        }

        if existing.roles.admin != new_escrow.roles.admin {
            return Err(EscrowError::AdminAddressCannotBeChanged);
        }

        if existing.roles.platform != new_escrow.roles.platform {
            return Err(EscrowError::PlatformAddressCannotBeChanged);
        }

        if existing.flags.disputed {
            return Err(EscrowError::EscrowOpenedForDisputeResolution);
        }

        if new_escrow.flags.released || new_escrow.flags.disputed || new_escrow.flags.resolved {
            return Err(EscrowError::FlagsMustBeFalse);
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
                return Err(EscrowError::EscrowPropertiesMismatch);
            }
        }
        validate_admin_role_overlap(&new_escrow.roles)?;
    }

    Ok(())
}

#[inline]
pub fn validate_add_milestones_conditions(
    existing_escrow: &Escrow,
    admin: &Address,
    new_milestones: &Vec<Milestone>,
) -> Result<(), EscrowError> {
    if admin != &existing_escrow.roles.admin {
        return Err(EscrowError::OnlyAdminAddressExecuteThisFunction);
    }
    if existing_escrow.flags.disputed {
        return Err(EscrowError::EscrowOpenedForDisputeResolution);
    }
    if existing_escrow.flags.released {
        return Err(EscrowError::EscrowAlreadyReleased);
    }
    if existing_escrow.flags.resolved {
        return Err(EscrowError::EscrowAlreadyResolved);
    }
    if new_milestones.is_empty() {
        return Err(EscrowError::NoMilestoneDefined);
    }
    if existing_escrow.milestones.len() + new_milestones.len() > 50 {
        return Err(EscrowError::TooManyMilestones);
    }
    for milestone in new_milestones.iter() {
        if milestone.approvals.quorum == 0 {
            return Err(EscrowError::QuorumCannotBeZero);
        }
        if milestone.approvals.approval_count > 0 {
            return Err(EscrowError::FlagsMustBeFalse);
        }
    }
    Ok(())
}

#[inline]
pub fn validate_escrow_property_change_conditions(
    existing_escrow: &Escrow,
    new_escrow: &Escrow,
    admin: &Address,
    contract_balance: i128,
) -> Result<(), EscrowError> {
    validate_escrow_conditions(
        Some(existing_escrow),
        new_escrow,
        Some(admin),
        Some(contract_balance),
        false,
    )
}

#[inline]
pub fn validate_initialize_escrow_conditions(
    e: &Env,
    escrow_properties: Escrow,
) -> Result<(), EscrowError> {
    if e.storage().persistent().has(&DataKey::Escrow) {
        return Err(EscrowError::EscrowAlreadyInitialized);
    }
    validate_escrow_conditions(None, &escrow_properties, None, None, true)
}

#[inline]
pub fn validate_fund_escrow_conditions(
    amount: i128,
    balance: i128,
    stored_escrow: &Escrow,
    expected_escrow: &Escrow,
) -> Result<(), EscrowError> {
    if amount <= 0 {
        return Err(EscrowError::AmountCannotBeZero);
    }

    if !stored_escrow.eq(&expected_escrow) {
        return Err(EscrowError::EscrowPropertiesMismatch);
    }

    if balance < amount {
        return Err(EscrowError::InsufficientFundsForEscrowFunding);
    }

    Ok(())
}
