use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct Escrow {
    pub engagement_id: String,
    pub title: String,
    pub roles: Roles,
    pub description: String,
    pub amount: i128,
    pub platform_fee: i128,
    pub milestones: Vec<Milestone>,
    pub flags: Flags,
    pub trustline: Trustline,
    pub receiver_memo: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Milestone {
    pub description: String,
    pub status: String,
    pub evidence: String,
    pub approved: bool,
}

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct Roles {
    pub approver: Address,
    pub service_provider: Address,
    pub platform_address: Address,
    pub release_signer: Address,
    pub dispute_resolver: Address,
    pub receiver: Address,
}

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct Flags {
    pub disputed: bool,
    pub released: bool,
    pub resolved: bool,
}

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct Trustline {
    pub address: Address,
    pub decimals: u32,
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
    Admin,
    Escrow,
}
