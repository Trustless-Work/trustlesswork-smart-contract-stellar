use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct Escrow {
    pub engagement_id: String,
    pub title: String,
    pub roles: Roles,
    pub description: String,
    pub amount: i128,
    pub platform_fee: u32,
    pub milestones: Vec<Milestone>,
    pub dispute: Dispute,
    pub released: bool,
    pub trustline: Trustline,
    pub receiver_memo: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneApprovals {
    pub quorum: u32,
    pub approval_count: u32,
    pub approvers: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Milestone {
    pub description: String,
    pub status: String,
    pub evidence: String,
    pub approvals: MilestoneApprovals,
}

#[contracttype]
#[derive(Clone)]
pub struct MilestoneStatusUpdate {
    pub milestone_index: u32,
    pub new_status: String,
    pub new_evidence: Option<String>,
}

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct Roles {
    pub approvers: Vec<Address>,
    pub service_providers: Vec<Address>,
    pub platform: Address,
    pub release_signers: Vec<Address>,
    pub dispute_resolvers: Vec<Address>,
    pub receiver: Address,
    pub admin: Address,
    pub observers: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dispute {
    pub is_disputed: bool,
    pub reason: String,
    pub resolved: bool,
}

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct Trustline {
    pub address: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct AddressBalance {
    pub address: Address,
    pub balance: i128,
    pub trustline_decimals: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Escrow,
    Admin,
}
