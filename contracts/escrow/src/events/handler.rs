use soroban_sdk::{contractevent, Address, BytesN, String, Vec};

use crate::storage::types::{DistributionEntry, MilestoneStatusEntry};

#[contractevent(topics = ["tw_init"])]
#[derive(Clone)]
pub struct InitEsc {
    #[topic]
    pub engagement_id: String,
    pub amount: i128,
    pub platform_fee: u32,
    pub trustline: Address,
    pub receiver: Address,
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
    pub receiver: Address,
    pub amount: i128,
    pub platform_fee: i128,
    pub trustless_work_fee: i128,
    pub net_amount: i128,
}

#[contractevent(topics = ["tw_update"])]
#[derive(Clone)]
pub struct EscrowUpdated {
    #[topic]
    pub engagement_id: String,
    pub admin: Address,
    /// Names of the top-level Escrow fields whose value changed. Empty when
    /// the update stored an escrow identical to the one it replaced.
    pub changed_fields: Vec<String>,
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

#[contractevent(topics = ["tw_dispute"])]
#[derive(Clone)]
pub struct EscrowDisputed {
    #[topic]
    pub engagement_id: String,
    pub signer: Address,
    pub reason: String,
}

#[contractevent(topics = ["tw_disp_resolve"])]
#[derive(Clone)]
pub struct DisputeResolved {
    #[topic]
    pub engagement_id: String,
    pub dispute_resolver: Address,
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
    /// Indices of the existing milestones that were updated, in call order.
    pub updated_indices: Vec<u32>,
    /// sha256 of each newly added milestone's description, in append order —
    /// hashed rather than echoed to keep the event small.
    pub added_description_hashes: Vec<BytesN<32>>,
}

#[contractevent(topics = ["tw_ttl_extend"])]
#[derive(Clone)]
pub struct TtlExtended {
    #[topic]
    pub engagement_id: String,
    pub admin: Address,
    pub ledgers_to_extend: u32,
}
