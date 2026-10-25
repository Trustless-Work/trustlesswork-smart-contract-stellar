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
    pub flags: Flags,
    pub trustline: Trustline,
    pub receiver_memo: u32,
    /// DeFindex vault that holds the funds while they are in escrow.
    /// `None` keeps the funds in the escrow itself.
    pub vault: Option<Address>,
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
    pub platform: Address,
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
}

#[contracttype]
#[derive(Clone)]
pub struct AddressBalance {
    pub address: Address,
    pub balance: i128,
    pub trustline_decimals: u32,
}

/// The escrow's position in its DeFindex vault.
#[contracttype]
#[derive(Clone)]
pub struct VaultPosition {
    pub vault: Address,
    pub shares: i128,
    pub underlying: i128,
}

/// Mirrors DeFindex's `Strategy`, to decode `get_assets`.
#[contracttype]
#[derive(Clone)]
pub struct Strategy {
    pub address: Address,
    pub name: String,
    pub paused: bool,
}

/// Mirrors DeFindex's `AssetStrategySet`, to decode `get_assets`.
#[contracttype]
#[derive(Clone)]
pub struct AssetStrategySet {
    pub address: Address,
    pub strategies: Vec<Strategy>,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Escrow,
    Admin,
    FundedAmount,
}
