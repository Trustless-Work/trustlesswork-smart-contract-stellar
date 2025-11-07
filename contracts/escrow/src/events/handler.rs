use crate::storage::types::Escrow;
use soroban_sdk::{contractevent, String};

#[contractevent(topics = ["tw_init"], data_format = "vec")]
#[derive(Clone)]
pub struct InitEsc {
    pub escrow: Escrow,
}

#[contractevent(topics = ["tw_fund"], data_format = "vec")]
#[derive(Clone)]
pub struct FundEsc {
    pub signer: soroban_sdk::Address,
    pub amount: i128,
}

#[contractevent(topics = ["tw_release"], data_format = "single-value")]
#[derive(Clone)]
pub struct DisEsc {
    pub release_signer: soroban_sdk::Address,
}

#[contractevent(topics = ["tw_update"], data_format = "vec")]
#[derive(Clone)]
pub struct ChgEsc {
    pub platform: soroban_sdk::Address,
    pub engagement_id: String,
    pub new_escrow_properties: Escrow,
}

// Milestones
#[contractevent(topics = ["tw_ms_change"], data_format = "vec")]
#[derive(Clone)]
pub struct MilestoneStatusChanged {
    pub escrow: Escrow,
}

#[contractevent(topics = ["tw_ms_approve"], data_format = "vec")]
#[derive(Clone)]
pub struct MilestoneApproved {
    pub escrow: Escrow,
}

// Disputes
#[contractevent(topics = ["tw_disp_resolve"], data_format = "vec")]
#[derive(Clone)]
pub struct DisputeResolved {
    pub escrow: Escrow,
}

#[contractevent(topics = ["tw_dispute"], data_format = "vec")]
#[derive(Clone)]
pub struct EscrowDisputed {
    pub escrow: Escrow,
}

// Admin / TTL
#[contractevent(topics = ["tw_ttl_extend"], data_format = "vec")]
#[derive(Clone)]
pub struct ExtTtlEvt {
    pub platform: soroban_sdk::Address,
    pub ledgers_to_extend: u32,
}
