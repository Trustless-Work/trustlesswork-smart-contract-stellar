use crate::storage::types::{
    DistributionEntry, EscrowPropertyChanges, MilestoneAddedEntry, MilestonePayout,
    MilestoneStatusEntry, MilestoneUpdatedEntry,
};
use soroban_sdk::{contractevent, Address, Bytes, BytesN, Env, String, Vec};

/// Longest free-text field the contract accepts (evidence and description are
/// both capped at 500 bytes by the validators). Sizing the copy buffer to this
/// bound lets us hash any accepted string on-stack, without an allocator.
const MAX_HASHABLE_LEN: usize = 500;

/// SHA-256 of a Soroban `String`, used to prove *which* free-text content a
/// value carried without bloating the event with the raw bytes.
///
/// Callers must only pass strings that have already cleared the validators'
/// length checks (evidence and description are both bounded at
/// `MAX_HASHABLE_LEN`); the events that use this are built after validation
/// succeeds, so the on-stack buffer is always large enough.
pub fn hash_string(e: &Env, value: &String) -> BytesN<32> {
    let len = value.len() as usize;
    let mut buf = [0u8; MAX_HASHABLE_LEN];
    value.copy_into_slice(&mut buf[..len]);
    let bytes = Bytes::from_slice(e, &buf[..len]);
    e.crypto().sha256(&bytes).to_bytes()
}

#[contractevent(topics = ["tw_init"])]
#[derive(Clone)]
pub struct InitEsc {
    #[topic]
    pub engagement_id: String,
    pub milestone_count: u32,
    pub total_amount: i128,
}

#[contractevent(topics = ["tw_fund"])]
#[derive(Clone)]
pub struct FundEsc {
    #[topic]
    pub engagement_id: String,
    pub funder: Address,
    pub amount: i128,
    pub funded_total: i128,
}

#[contractevent(topics = ["tw_release"])]
#[derive(Clone)]
pub struct ReleaseEsc {
    #[topic]
    pub engagement_id: String,
    pub release_signer: Address,
    pub payouts: Vec<MilestonePayout>,
}

#[contractevent(topics = ["tw_update"])]
#[derive(Clone)]
pub struct EscrowUpdated {
    #[topic]
    pub engagement_id: String,
    pub admin: Address,
    pub changes: EscrowPropertyChanges,
}

#[contractevent(topics = ["tw_ms_change"])]
#[derive(Clone)]
pub struct MilestoneStatusChanged {
    #[topic]
    pub engagement_id: String,
    pub service_provider: Address,
    pub updates: Vec<MilestoneStatusEntry>,
}

#[contractevent(topics = ["tw_ms_approve"])]
#[derive(Clone)]
pub struct MilestonesApproved {
    #[topic]
    pub engagement_id: String,
    pub approver: Address,
    pub milestone_indices: Vec<u32>,
}

#[contractevent(topics = ["tw_ms_dispute"])]
#[derive(Clone)]
pub struct MilestonesDisputed {
    #[topic]
    pub engagement_id: String,
    pub signer: Address,
    pub reason: String,
    pub milestone_indices: Vec<u32>,
}

#[contractevent(topics = ["tw_disp_resolve"])]
#[derive(Clone)]
pub struct DisputeResolved {
    #[topic]
    pub engagement_id: String,
    pub dispute_resolver: Address,
    pub milestone_indices: Vec<u32>,
    pub platform_fee: i128,
    pub trustless_work_fee: i128,
    pub distributions: Vec<DistributionEntry>,
}

#[contractevent(topics = ["tw_withdraw"])]
#[derive(Clone)]
pub struct FundsWithdrawn {
    #[topic]
    pub engagement_id: String,
    pub dispute_resolver: Address,
    pub platform_fee: i128,
    pub trustless_work_fee: i128,
    pub distributions: Vec<DistributionEntry>,
}

#[contractevent(topics = ["tw_ms_manage"])]
#[derive(Clone)]
pub struct MilestonesManaged {
    #[topic]
    pub engagement_id: String,
    pub admin: Address,
    pub added_count: u32,
    pub updated_count: u32,
    /// The milestones appended by this call (final index + key fields).
    pub added: Vec<MilestoneAddedEntry>,
    /// The in-place edits this call applied (index + changed fields only).
    pub updated: Vec<MilestoneUpdatedEntry>,
}

#[contractevent(topics = ["tw_ttl_extend"])]
#[derive(Clone)]
pub struct TtlExtended {
    #[topic]
    pub engagement_id: String,
    pub admin: Address,
    pub ledgers_to_extend: u32,
}
