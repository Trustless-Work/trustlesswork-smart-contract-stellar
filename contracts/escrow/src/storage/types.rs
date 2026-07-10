use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

#[contracttype]
#[derive(Clone)]
pub struct MilestoneStatusEntry {
    pub index: u32,
    pub status: String,
}

#[contracttype]
#[derive(Clone)]
pub struct DistributionEntry {
    pub address: Address,
    pub amount: i128,
}

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

/// Cross-chain payout target, set only by the receiver.
///
/// `max_fee` is the CCTP Forwarding Service ceiling (Stellar 7-decimal
/// stroops) the receiver approves for the burn — sized by the API from a
/// live Circle fee quote at the time this is set, not a value the API's
/// caller supplies. Living here (not as a `release_funds` argument) keeps
/// the release signer unable to influence it: only the receiver's own
/// signature on this call authorizes it.
#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct CrossChainDestination {
    pub destination_domain: u32,
    pub mint_recipient: BytesN<32>,
    pub max_fee: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneApprovals {
    pub target: u32,
    pub approval_count: u32,
    pub approved_by: Vec<Address>,
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
#[derive(Clone)]
pub struct MilestoneUpdate {
    pub index: u32,
    pub new_description: Option<String>,
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
    FundedAmount,
    Reentrancy,
    ApprovedWasmHash,
    CrossChainDestination,
}
