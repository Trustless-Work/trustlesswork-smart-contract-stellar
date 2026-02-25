use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct Escrow {
    pub engagement_id: String,
    pub title: String,
    pub description: String,
    pub roles: Roles,
    pub platform_fee: u32,
    pub milestones: Vec<Milestone>,
    pub trustline: Trustline,
    pub receiver_memo: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Milestone {
    pub description: String,
    pub status: String,
    pub evidence: String,
    pub amount: i128,
    pub flags: Flags,
    pub receiver: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneUpdate {
    pub index: u32,
    pub status: String,
    pub evidence: Option<String>,
}

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct Roles {
    pub approver: Address,
    pub service_provider: Address,
    pub platform_address: Address,
    pub release_signer: Address,
    pub dispute_resolver: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flags {
    pub disputed: bool,
    pub released: bool,
    pub resolved: bool,
    pub approved: bool,
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
}
