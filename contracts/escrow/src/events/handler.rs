use soroban_sdk::{contractevent, Address, String};

#[contractevent(topics = ["tw_init"], data_format = "single-value")]
#[derive(Clone)]
pub struct InitEsc {
    pub engagement_id: String,
}

#[contractevent(topics = ["tw_fund"], data_format = "vec")]
#[derive(Clone)]
pub struct FundEsc {
    pub signer: Address,
    pub amount: i128,
}

#[contractevent(topics = ["tw_release"], data_format = "single-value")]
#[derive(Clone)]
pub struct DisEsc {
    pub release_signer: Address,
}

#[contractevent(topics = ["tw_update"], data_format = "vec")]
#[derive(Clone)]
pub struct ChgEsc {
    pub platform: Address,
    pub engagement_id: String,
}

// Milestones
#[contractevent(topics = ["tw_ms_change"], data_format = "single-value")]
#[derive(Clone)]
pub struct MilestoneStatusChanged {
    pub engagement_id: String,
}

#[contractevent(topics = ["tw_ms_approve"], data_format = "single-value")]
#[derive(Clone)]
pub struct MilestonesApproved {
    pub engagement_id: String,
}

// Disputes
#[contractevent(topics = ["tw_disp_resolve"], data_format = "single-value")]
#[derive(Clone)]
pub struct DisputeResolved {
    pub engagement_id: String,
}

#[contractevent(topics = ["tw_dispute"], data_format = "single-value")]
#[derive(Clone)]
pub struct EscrowDisputed {
    pub engagement_id: String,
}

// Milestones managed (add or update)
#[contractevent(topics = ["tw_ms_manage"], data_format = "single-value")]
#[derive(Clone)]
pub struct MilestonesManaged {
    pub engagement_id: String,
}

// Admin / TTL
#[contractevent(topics = ["tw_ttl_extend"], data_format = "vec")]
#[derive(Clone)]
pub struct ExtTtlEvt {
    pub platform: Address,
    pub ledgers_to_extend: u32,
}
