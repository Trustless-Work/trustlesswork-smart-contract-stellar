# Trustless Work — Multi-Release Escrow Contract Documentation

> **Branch:** `multi-release-develop-v2`  
> **SDK:** Soroban SDK `26.0.0` / Stellar Soroban (Rust, `#![no_std]`)  
> **Build target:** `wasm32v1-none --release`  
> **Contract name:** `EscrowContract`

---

## Table of Contents

1. [Overview](#1-overview)
2. [Architecture & File Structure](#2-architecture--file-structure)
3. [Data Types (Storage)](#3-data-types-storage)
4. [Roles System](#4-roles-system)
5. [Contract Entry Points (Public Functions)](#5-contract-entry-points-public-functions)
   - 5.1 [Constructor](#51-constructor)
   - 5.2 [tw_new_multi_release_escrow](#52-tw_new_multi_release_escrow)
   - 5.3 [initialize_escrow](#53-initialize_escrow)
   - 5.4 [fund_escrow](#54-fund_escrow)
   - 5.5 [release_funds](#55-release_funds)
   - 5.6 [update_escrow](#56-update_escrow)
   - 5.7 [manage_milestones](#57-manage_milestones)
   - 5.8 [change_milestone_status](#58-change_milestone_status)
   - 5.9 [approve_milestones](#59-approve_milestones)
   - 5.10 [approve_and_release_milestones](#510-approve_and_release_milestones)
   - 5.11 [dispute_milestones](#511-dispute_milestones)
   - 5.12 [resolve_dispute](#512-resolve_dispute)
   - 5.13 [withdraw_remaining_funds](#513-withdraw_remaining_funds)
   - 5.14 [get_escrow](#514-get_escrow)
   - 5.15 [get_escrow_by_contract_id](#515-get_escrow_by_contract_id)
   - 5.16 [get_multiple_escrow_balances](#516-get_multiple_escrow_balances)
   - 5.17 [extend_contract_ttl](#517-extend_contract_ttl)
6. [Core Managers](#6-core-managers)
   - 6.1 [EscrowManager](#61-escrowmanager)
   - 6.2 [MilestoneManager](#62-milestonemanager)
   - 6.3 [DisputeManager](#63-disputemanager)
7. [Fee System](#7-fee-system)
8. [Validation Rules](#8-validation-rules)
9. [Events](#9-events)
10. [Error Codes](#10-error-codes)
11. [Storage Layout](#11-storage-layout)
12. [Lifecycle Flow](#12-lifecycle-flow)
13. [Security Properties](#13-security-properties)
14. [Differences vs Single-Release](#14-differences-vs-single-release)

---

## 1. Overview

This is a **multi-release escrow** smart contract deployed on the **Stellar Soroban** blockchain. It holds stablecoins (e.g. USDC) on behalf of parties and releases them **per-milestone**. Each milestone has its own:

- **amount** (funds allocated to that milestone)
- **receiver** (who gets paid when this milestone is released)
- **approval tracking** (multi-approver threshold model)
- **dispute state** (can be disputed/resolved independently)

The contract is designed as a **factory + instance** pattern: a parent (factory) contract deploys child escrow contracts via `tw_new_multi_release_escrow`. Each deployed child contract holds exactly one escrow.

**Key capability:** Multiple milestones can be funded under one escrow, and each can be approved, disputed, and released independently.

---

## 2. Architecture & File Structure

```
contracts/escrow/
├── Cargo.toml                       # Package: soroban-sdk 26.0.0
└── src/
    ├── lib.rs                       # Module tree root, no_std
    ├── contract.rs                  # EscrowContract — all public entry points
    ├── error.rs                     # EscrowError, ReleaseError, MilestoneError enums
    ├── storage/
    │   └── types.rs                 # All #[contracttype] structs and DataKey enum
    ├── core/
    │   ├── escrow.rs                # EscrowManager — fund, release, update, query
    │   ├── milestone.rs             # MilestoneManager — status change, approvals
    │   ├── dispute.rs               # DisputeManager — dispute, resolve, withdraw
    │   └── validators/
    │       ├── escrow.rs            # Escrow init/fund/release/update validators
    │       ├── milestone.rs         # Milestone status-change/approval validators
    │       └── dispute.rs           # Dispute/resolve/withdraw validators
    ├── events/
    │   └── handler.rs               # All #[contractevent] structs
    └── modules/
        ├── fee/
        │   ├── calculator.rs        # FeeCalculator — basis-point fee math
        │   └── distribution.rs      # calculate_and_distribute_fees
        └── math/
            ├── basic.rs             # BasicMath — safe_add, safe_sub, safe_mul, safe_div
            └── safe.rs              # SafeMath — safe_mul_div (overflow-checked)
```

---

## 3. Data Types (Storage)

All types are decorated with `#[contracttype]` which makes them ABI-serializable on Soroban.

### `Escrow`
The root storage struct, stored under `DataKey::Escrow`.

```rust
pub struct Escrow {
    pub engagement_id: String,    // Unique ID (max 100 chars)
    pub title: String,            // Human-readable title (max 100 chars)
    pub roles: Roles,             // All role assignments
    pub description: String,      // Description (max 500 chars)
    pub platform_fee: u32,        // Platform fee in basis points (e.g. 300 = 3%)
    pub milestones: Vec<Milestone>, // Ordered list of milestones (max 50)
    pub trustline: Trustline,     // The token contract address
    pub receiver_memo: u32,       // Optional memo for receiver identification
}
```

### `Milestone`
Each milestone is independently releasable.

```rust
pub struct Milestone {
    pub description: String,       // What work this milestone covers (max 500 chars)
    pub status: String,            // Free-text status string (max 50 chars)
    pub evidence: String,          // URL or text proof of completion (max 500 chars)
    pub approvals: MilestoneApprovals, // Threshold approval tracking
    pub amount: i128,              // Token amount allocated (must be > 0)
    pub dispute: Dispute,          // Dispute state for this milestone
    pub released: bool,            // True once funds for this milestone have been sent
    pub receiver: Address,         // Who receives payment for this milestone
}
```

### `MilestoneApprovals`
Threshold-based approval system per milestone.

```rust
pub struct MilestoneApprovals {
    pub target: u32,              // Number of approvals required (must be > 0, <= approvers.len())
    pub approval_count: u32,      // Current count of unique approvals received
    pub approved_by: Vec<Address>, // List of approvers who have already voted
}
```

A milestone is considered **approved** when `approval_count >= target`.

### `Dispute`
Per-milestone dispute state.

```rust
pub struct Dispute {
    pub is_disputed: bool,  // True while dispute is open and unresolved
    pub reason: String,     // Reason text provided when dispute was opened (max 500 chars)
    pub resolved: bool,     // True once dispute has been resolved (terminal state)
}
```

### `Roles`
All actor addresses for the escrow.

```rust
pub struct Roles {
    pub approvers: Vec<Address>,         // Can approve milestones (max 5)
    pub service_providers: Vec<Address>, // Can change milestone status (max 5)
    pub platform: Address,               // Receives platform fee
    pub release_signers: Vec<Address>,   // Can trigger fund release (max 5)
    pub dispute_resolvers: Vec<Address>, // Can open resolution / resolve disputes (max 5)
    pub admin: Address,                  // Can update escrow properties and manage milestones
    pub observers: Vec<Address>,         // Read-only observers (max 5)
}
```

> **Note:** `receiver` is NOT in `Roles` for multi-release. Each `Milestone` carries its own `receiver`.

### `Trustline`
```rust
pub struct Trustline {
    pub address: Address, // The Stellar asset contract (e.g. USDC SEP-41 token)
}
```

### `MilestoneStatusUpdate`
Used as input for batch status changes.

```rust
pub struct MilestoneStatusUpdate {
    pub milestone_index: u32,
    pub new_status: String,
    pub new_evidence: Option<String>,
}
```

### `MilestoneUpdate`
Used as input for `manage_milestones` to modify existing milestones.

```rust
pub struct MilestoneUpdate {
    pub index: u32,
    pub new_description: Option<String>,
    pub new_amount: Option<i128>,
}
```

> Milestone amount can only be changed when the contract has zero balance (`FundedAmount == 0`).

### `MilestonePayout`
Returned in events to describe each milestone's payment breakdown.

```rust
pub struct MilestonePayout {
    pub index: u32,
    pub receiver: Address,
    pub amount: i128,          // Gross milestone amount
    pub platform_fee: i128,
    pub trustless_work_fee: i128,
    pub net_amount: i128,      // Amount the receiver actually receives
}
```

### `AddressBalance`
Result struct for `get_multiple_escrow_balances`.

```rust
pub struct AddressBalance {
    pub address: Address,
    pub balance: i128,
    pub trustline_decimals: u32,
}
```

### `DataKey`
Soroban persistent storage keys.

```rust
pub enum DataKey {
    Escrow,           // The Escrow struct
    Admin,            // Temporary admin address used during initialization
    FundedAmount,     // Running total of tokens transferred in (i128)
    Reentrancy,       // Reentrancy guard flag (bool)
    ApprovedWasmHash, // Approved WASM hash for child deployment
}
```

---

## 4. Roles System

| Role | Who they are | What they can do |
|------|-------------|-----------------|
| `admin` | Contract deployer / platform backend | `initialize_escrow`, `update_escrow`, `manage_milestones`, `extend_contract_ttl` |
| `approvers` | Client(s) or designated reviewers | `approve_milestones`, `approve_and_release_milestones`, `dispute_milestones` |
| `service_providers` | Freelancer/contractor side | `change_milestone_status`, `dispute_milestones` |
| `release_signers` | Platform or designated signer | `release_funds`, `approve_and_release_milestones` |
| `dispute_resolvers` | Neutral arbitrator | `resolve_dispute`, `withdraw_remaining_funds` |
| `platform` | Fee receiver address | Receives `platform_fee` on every release |
| `observers` | Auditors/watchers | No on-chain write access; for off-chain indexing |
| `milestone.receiver` | Payee per milestone | Receives `net_amount` when milestone is released; can `dispute_milestones` |

**Role constraints enforced at initialization:**
- `admin` cannot overlap with `approvers`, `service_providers`, `release_signers`, or `dispute_resolvers`
- `dispute_resolvers` cannot overlap with `approvers`, `service_providers`, or `release_signers`
- No duplicate addresses within any role list
- Each role list is capped at **5 members** maximum
- `admin` and `platform` addresses **cannot be changed** after initialization (immutable)

---

## 5. Contract Entry Points (Public Functions)

### 5.1 Constructor
```rust
pub fn __constructor(e: Env, admin: Address, approved_wasm_hash: BytesN<32>)
```
Called once at contract deployment. Stores `admin` and `approved_wasm_hash` in persistent storage with TTL extended to 1 year (31,536,000 ledgers). Both are removed from storage after `initialize_escrow` is called, making the contract permanently settled.

### 5.2 `tw_new_multi_release_escrow`
```rust
pub fn tw_new_multi_release_escrow(
    env: Env,
    signer: Address,
    wasm_hash: BytesN<32>,
    salt: BytesN<32>,
    init_fn: Symbol,
    init_args: Vec<Val>,
    constructor_args: Vec<Val>,
) -> Result<(Address, Val), EscrowError>
```
**Factory deployment function.** Deploys a new child escrow contract and calls its init function in one transaction.

- Fails if the calling contract already has an escrow initialized (`EscrowAlreadyInitialized`)
- Validates `wasm_hash` matches the stored `approved_wasm_hash` (prevents deploying unapproved code)
- Requires `signer.require_auth()`
- Deploys via `env.deployer().with_address(deployer, salt).deploy_v2(wasm_hash, constructor_args)`
- Returns `(deployed_address, init_return_value)`

### 5.3 `initialize_escrow`
```rust
pub fn initialize_escrow(e: &Env, escrow_properties: Escrow) -> Result<Escrow, EscrowError>
```
Sets up the escrow with all its configuration. Can only be called once.

- Requires `stored_admin.require_auth()` (the address stored in `DataKey::Admin`)
- After success, **removes** `DataKey::Admin` and `DataKey::ApprovedWasmHash` from storage permanently
- Extends TTL of `DataKey::Escrow` to 1 year
- Emits `InitEsc` event with `engagement_id`, `milestone_count`, `total_amount`

**Validation:** Full escrow validation (see [Section 8](#8-validation-rules)).

### 5.4 `fund_escrow`
```rust
pub fn fund_escrow(
    e: &Env,
    signer: Address,
    expected_escrow: Escrow,
    amount: i128,
) -> Result<(), EscrowError>
```
Transfers tokens from `signer` into the contract. The `expected_escrow` parameter is a **security check** — the caller must pass the exact current escrow state to prevent race conditions.

- Validates `amount > 0`
- Validates `stored_escrow == expected_escrow` (mismatch → `EscrowPropertiesMismatch`)
- Validates `signer` token balance ≥ `amount`
- Calls `token_client.transfer(signer, contract, amount)`
- Increments `DataKey::FundedAmount`
- Emits `FundEsc` event with `engagement_id`, `funder`, `amount`, `funded_total`

### 5.5 `release_funds`
```rust
pub fn release_funds(
    e: &Env,
    release_signer: Address,
    trustless_work_address: Address,
    milestone_indices: Vec<u32>,
) -> Result<(), ReleaseError>
```
Releases funds for specific milestones. Each milestone in `milestone_indices` must be individually approved and undisputed.

- Only callable by a `release_signer`
- Each milestone must: be approved (`approval_count >= target`), not be disputed, not already released, not already dispute-resolved
- No duplicate indices allowed
- Marks milestones as `released = true` **before** transferring tokens (effects-before-interactions pattern)
- For each milestone: transfers `trustless_work_fee` → `trustless_work_address`, `platform_fee` → `platform`, `net_amount` → `milestone.receiver`
- Emits `ReleaseEsc` event with full `payouts: Vec<MilestonePayout>`

### 5.6 `update_escrow`
```rust
pub fn update_escrow(
    e: &Env,
    admin_address: Address,
    escrow_properties: Escrow,
) -> Result<Escrow, EscrowError>
```
Allows the `admin` to update escrow metadata and role assignments (but not milestones — use `manage_milestones` for that).

- Only callable by current `escrow.roles.admin`
- **Cannot change** `admin` address or `platform` address
- If contract has funds (balance > 0): cannot change `engagement_id`, `title`, `description`, `roles`, `platform_fee`, `trustline`, or `receiver_memo`
- Cannot be called if any milestone is currently disputed
- Preserves existing milestones (new `escrow_properties.milestones` field is ignored)
- Emits `EscrowUpdated` event

### 5.7 `manage_milestones`
```rust
pub fn manage_milestones(
    e: &Env,
    admin_address: Address,
    new_milestones: Vec<Milestone>,
    milestone_updates: Vec<MilestoneUpdate>,
) -> Result<Escrow, EscrowError>
```
Adds new milestones or updates descriptions/amounts of existing milestones.

- Only callable by `admin`
- Cannot call with both lists empty
- Cannot change milestone `amount` if contract has funds (`FundedAmount > 0`)
- `new_amount` must be > 0 (`AmountCannotBeZero`)
- New milestones: must have `amount > 0`, `approvals.target > 0`, all flags clear, `target <= approvers.len()`
- String limits enforced on inputs: new-milestone `description`/`evidence` ≤ 500 chars, `status` ≤ 50 chars, and `new_description` ≤ 500 chars (`StringTooLong`)
- Total milestone count cannot exceed 50
- Cannot be called if any milestone is disputed, already released, or dispute-resolved
- Emits `MilestonesManaged` event with counts plus the added/updated milestone details

### 5.8 `change_milestone_status`
```rust
pub fn change_milestone_status(
    e: Env,
    updates: Vec<MilestoneStatusUpdate>,
    service_provider: Address,
) -> Result<(), MilestoneError>
```
Allows service providers to update the status and evidence of milestones in a batch.

- Only callable by a `service_provider`
- Batch size: 1–50 updates
- Status string: 1–50 chars; evidence string: 0–500 chars
- Milestone index must be valid
- Emits `MilestoneStatusChanged` event

### 5.9 `approve_milestones`
```rust
pub fn approve_milestones(
    e: Env,
    milestone_indices: Vec<u32>,
    approver: Address,
) -> Result<(), MilestoneError>
```
Records an approver's vote for a set of milestones.

- Only callable by an address in `approvers`
- Each approver can only vote once per milestone (`ApproverAlreadyApprovedMilestone`)
- Cannot approve a milestone that has already reached its approval threshold
- Batch size: 1–50, no duplicates
- Appends `approver` to `milestone.approvals.approved_by` and increments `approval_count`
- Emits `MilestonesApproved` event

### 5.10 `approve_and_release_milestones`
```rust
pub fn approve_and_release_milestones(
    e: Env,
    signer: Address,
    trustless_work_address: Address,
    milestone_indices: Vec<u32>,
) -> Result<(), EscrowError>
```
Atomic combination: approve + release in one transaction. Convenience function for the common case where the same person is both approver and release signer.

- `signer` must be in BOTH `approvers` AND `release_signers` (`SignerMustBeApproverAndReleaseSigner`)
- Internally calls `approve_milestones_inner` then `release_funds_inner` (skips double `require_auth`)
- Emits both `MilestonesApproved` and `ReleaseEsc` events

### 5.11 `dispute_milestones`
```rust
pub fn dispute_milestones(
    e: Env,
    signer: Address,
    milestone_indices: Vec<u32>,
    reason: String,
) -> Result<(), EscrowError>
```
Opens a dispute on specific milestones.

- Authorized callers: `approvers`, `service_providers`, `platform`, `release_signers` (global roles, any milestone), or a milestone `receiver`
- A receiver without a global role may only dispute milestones **they are the receiver of** — every index in the batch must belong to them, so mixed batches including another receiver's milestone are rejected (`UnauthorizedToChangeDisputeFlag`)
- `dispute_resolvers` are **explicitly blocked** from opening disputes
- Each targeted milestone must not already be disputed, resolved, or released
- Reason string: max 500 chars
- Sets `milestone.dispute.is_disputed = true` and stores `reason`
- Emits `MilestonesDisputed` event

### 5.12 `resolve_dispute`
```rust
pub fn resolve_dispute(
    e: Env,
    dispute_resolver: Address,
    trustless_work_address: Address,
    milestone_indices: Vec<u32>,
    distributions: Map<Address, i128>,
) -> Result<(), EscrowError>
```
Resolves a dispute for specific milestones by distributing funds proportionally.

- Only callable by a `dispute_resolver`
- Protected by a reentrancy guard (`DataKey::Reentrancy`; re-entry fails with `FlagsMustBeFalse`)
- Each milestone in `milestone_indices` must have `is_disputed == true` and `resolved == false`; duplicate indices are rejected (`InvalidMilestoneIndex`)
- `distributions` sum must **exactly equal** the total amount of the specified milestones — partial settlement is rejected (`DistributionsMustEqualEscrowBalance`)
- Each amount in distributions must be > 0; max 50 entries
- Sets `dispute.resolved = true`, `dispute.is_disputed = false` for each resolved milestone
- Fees are calculated on the total and distributed proportionally among recipients (see [Section 7](#7-fee-system))
- Rounding remainder assigned to last recipient
- Emits `DisputeResolved` event

### 5.13 `withdraw_remaining_funds`
```rust
pub fn withdraw_remaining_funds(
    e: Env,
    dispute_resolver: Address,
    trustless_work_address: Address,
    distributions: Map<Address, i128>,
) -> Result<(), EscrowError>
```
Used when **all milestones** have been processed (released or dispute-resolved) and there are leftover funds in the contract.

- Protected by reentrancy guard (`DataKey::Reentrancy`)
- Requires all milestones to be either `released` or `dispute.resolved`
- Requires at least one milestone to have been disputed/resolved, **or** every milestone to be released — a fully-released escrow lets the resolver sweep surplus funds without any dispute
- `distributions` sum must **exactly equal** the contract token balance (`DistributionsMustEqualEscrowBalance`)
- Emits `FundsWithdrawn` event

### 5.14 `get_escrow`
```rust
pub fn get_escrow(e: &Env) -> Result<Escrow, EscrowError>
```
Read-only. Returns the stored `Escrow` struct.

### 5.15 `get_escrow_by_contract_id`
```rust
pub fn get_escrow_by_contract_id(e: &Env, contract_id: Address) -> Result<Escrow, EscrowError>
```
Cross-contract read. Calls `get_escrow` on another contract and returns its `Escrow`.

### 5.16 `get_multiple_escrow_balances`
```rust
pub fn get_multiple_escrow_balances(
    e: &Env,
    addresses: Vec<Address>,
) -> Result<Vec<AddressBalance>, EscrowError>
```
Batch balance query. For each address, retrieves the token balance held by that contract.

- Maximum 20 addresses per call
- Reads each contract's escrow to determine its trustline, then queries token balance

### 5.17 `extend_contract_ttl`
```rust
pub fn extend_contract_ttl(
    e: &Env,
    admin: Address,
    ledgers_to_extend: u32,
) -> Result<(), EscrowError>
```
Extends the TTL of `DataKey::Escrow` to prevent the storage entry from expiring on-chain.

- Only callable by `escrow.roles.admin`
- Uses minimum threshold of 17,280 ledgers (~1 day)
- Emits `TtlExtended` event

---

## 6. Core Managers

### 6.1 EscrowManager

Located in `src/core/escrow.rs`. Handles all escrow lifecycle operations.

**Key internal functions:**

- `get_receiver(milestone)` → `Address` — extracts receiver from milestone (not from roles)
- `release_funds_execute(e, trustless_work_address, milestone_indices, escrow)` — internal implementation shared by `release_funds` and `approve_and_release_milestones`; follows effects-before-interactions (marks released before token transfers)
- `get_escrow(e)` → reads from `DataKey::Escrow`
- `get_escrow_by_contract_id(e, contract_id)` → cross-contract invoke

### 6.2 MilestoneManager

Located in `src/core/milestone.rs`.

- `approve_milestones(e, indices, approver)` — calls `require_auth` then delegates to `approve_milestones_inner`
- `approve_milestones_inner(e, indices, approver)` — skips `require_auth` (used by `approve_and_release_milestones` which already authed)

### 6.3 DisputeManager

Located in `src/core/dispute.rs`.

- `dispute_milestones` — validates authorization, sets `is_disputed = true`
- `resolve_dispute` — validates disputed state, marks resolved, calls `calculate_and_distribute_fees`
- `withdraw_remaining_funds` — reentrancy-guarded, validates all-processed condition

---

## 7. Fee System

**Constants:**
```
TRUSTLESS_WORK_FEE_BPS = 30   // 0.30% to Trustless Work
BASIS_POINTS_DENOMINATOR = 10_000
```

**Formula for each release:**
```
trustless_work_fee = amount * 30 / 10_000
platform_fee = amount * platform_fee_bps / 10_000
receiver_amount = amount - trustless_work_fee - platform_fee
```

`platform_fee` is set by the platform deploying the contract (in basis points, max 9,900 bps = 99%).

**Constraint:** `platform_fee_bps + 30 ≤ 10_000` (total fees cannot exceed 100%).

**For dispute distributions:**

The fee is calculated on the total disputed amount and the net is distributed proportionally among recipients. Rounding dust (from integer division) is added to the last recipient so that `sum(all_transfers) == gross_amount` exactly.

```
distributable = total - trustless_work_fee - platform_fee
net_for_recipient_i = (recipient_amount_i * distributable) / total
remainder → last recipient
```

All arithmetic uses checked operations (`BasicMath`, `SafeMath`) and returns `EscrowError::Overflow`, `Underflow`, or `DivisionError` on failure.

---

## 8. Validation Rules

### Escrow Initialization
| Rule | Error |
|------|-------|
| `DataKey::Escrow` already exists | `EscrowAlreadyInitialized` |
| `DataKey::Admin` not found | `OnlyAdminAddressExecuteThisFunction` |
| `engagement_id` > 100 chars | `StringTooLong` |
| `title` > 100 chars | `StringTooLong` |
| `description` > 500 chars | `StringTooLong` |
| Milestone `description` > 500 chars | `StringTooLong` |
| Milestone `status` > 50 chars | `StringTooLong` |
| Milestone `evidence` > 500 chars | `StringTooLong` |
| `platform_fee` > 9,900 bps | `PlatformFeeTooHigh` |
| `platform_fee + 30 > 10,000` | `PlatformFeeTooHigh` |
| `approvers` list empty | `ApproversListEmpty` |
| `service_providers` list empty | `ServiceProvidersListEmpty` |
| `release_signers` list empty | `ReleaseSignersListEmpty` |
| `dispute_resolvers` list empty | `DisputeResolversListEmpty` |
| Any role list > 5 members | `RoleLimitExceeded` |
| Duplicate address within a role list | `DuplicateAddressInRole` |
| `dispute_resolver` in `approvers`/`service_providers`/`release_signers` | `DisputeResolverOverlapsWithOtherRole` |
| `admin` in any operational role | `AdminAddressOverlapsWithOtherRole` |
| Milestones count > 50 | `TooManyMilestones` |
| Milestone `amount <= 0` | `AmountCannotBeZero` |
| Milestone `approvals.target == 0` | `TargetCannotBeZero` |
| Milestone `target > approvers.len()` | `TargetExceedsApprovers` |
| Milestone has non-zero approval count or flags set | `FlagsMustBeFalse` |

### Release
| Rule | Error |
|------|-------|
| Caller not in `release_signers` | `OnlyReleaseSignerCanReleaseEarnings` |
| `milestone_indices` is empty | `ReleaseMilestonesEmpty` |
| Duplicate index in list | `DuplicateMilestoneIndex` |
| Index out of bounds | `InvalidMilestoneIndex` |
| Milestone `is_disputed == true` | `EscrowOpenedForDisputeResolution` |
| Milestone `dispute.resolved == true` | `EscrowAlreadyResolved` |
| Milestone not approved (`count < target`) | `EscrowNotCompleted` |
| Milestone already released | `MilestoneAlreadyReleased` |
| Contract balance < total release amount | `EscrowBalanceNotEnoughToSendEarnings` |

---

## 9. Events

All events are emitted via the `#[contractevent]` macro with Soroban's structured event system. The first field marked `#[topic]` becomes a filterable topic.

| Event Struct | Topic | Fields |
|---|---|---|
| `InitEsc` | `tw_init`, `engagement_id` | `milestone_count`, `total_amount` |
| `FundEsc` | `tw_fund`, `engagement_id` | `funder`, `amount`, `funded_total` |
| `ReleaseEsc` | `tw_release`, `engagement_id` | `release_signer`, `payouts: Vec<MilestonePayout>` |
| `EscrowUpdated` | `tw_update`, `engagement_id` | `admin`, `changes: EscrowPropertyChanges` |
| `MilestoneStatusChanged` | `tw_ms_change`, `engagement_id` | `service_provider`, `updates: Vec<MilestoneStatusEntry>` |
| `MilestonesApproved` | `tw_ms_approve`, `engagement_id` | `approver`, `milestone_indices` |
| `MilestonesDisputed` | `tw_ms_dispute`, `engagement_id` | `signer`, `reason`, `milestone_indices` |
| `DisputeResolved` | `tw_disp_resolve`, `engagement_id` | `dispute_resolver`, `milestone_indices`, `platform_fee`, `trustless_work_fee`, `distributions` |
| `FundsWithdrawn` | `tw_withdraw`, `engagement_id` | `dispute_resolver`, `platform_fee`, `trustless_work_fee`, `distributions` |
| `MilestonesManaged` | `tw_ms_manage`, `engagement_id` | `admin`, `added_count`, `updated_count`, `added: Vec<MilestoneAddedEntry>`, `updated: Vec<MilestoneUpdatedEntry>` |
| `TtlExtended` | `tw_ttl_extend`, `engagement_id` | `admin`, `ledgers_to_extend` |

### 9.1 Audit-trail payloads

Events carry **what changed**, not just that something changed, so an
events-only indexer can reconstruct history without diffing storage snapshots
(historical state is archived and may be unavailable).

**`MilestoneStatusEntry`** — one per updated milestone in `MilestoneStatusChanged`:

| Field | Type | Meaning |
|---|---|---|
| `index` | `u32` | Milestone index |
| `status` | `String` | New status |
| `evidence_hash` | `Option<BytesN<32>>` | SHA-256 of the evidence, or `None` if this update left the evidence untouched |

**`EscrowPropertyChanges`** — carried by `EscrowUpdated`:

| Field | Type | Meaning |
|---|---|---|
| `engagement_id`, `title`, `description`, `platform_fee`, `roles`, `trustline`, `receiver_memo` | `bool` | `true` when that property changed |
| `old_platform_fee` / `new_platform_fee` | `u32` | Before/after values (bps) |

`admin` and `platform` are absent: the contract forbids changing them
(`AdminAddressCannotBeChanged`, `PlatformAddressCannotBeChanged`).

**`MilestoneAddedEntry` / `MilestoneUpdatedEntry`** — carried by `MilestonesManaged`:

| Struct | Fields | Meaning |
|---|---|---|
| `MilestoneAddedEntry` | `index: u32`, `amount: i128`, `description_hash: BytesN<32>` | An appended milestone, at its final index |
| `MilestoneUpdatedEntry` | `index: u32`, `new_amount: Option<i128>`, `new_description_hash: Option<BytesN<32>>` | An in-place edit; each field is `Some` only when it changed |

**Why hashes instead of raw text.** Free-text fields (evidence, description) are
capped at 500 bytes; hashing keeps event payloads at a fixed 32 bytes while still
proving *which* content was recorded. To verify a claim, hash the presented text
and compare it to the value in the event — a mismatch proves the content was
altered. Note this is the hash of the field **content**, not the transaction hash.

**Ordering guarantee.** Hashing runs only *after* validation succeeds, so an
over-long string returns `StringTooLong` instead of trapping in the hasher.

---

## 10. Error Codes

### `EscrowError`
| Code | Name | Meaning |
|------|------|---------|
| 1 | `EscrowAlreadyInitialized` | `initialize_escrow` called on an already-initialized contract |
| 2 | `EscrowNotFound` | Storage key `DataKey::Escrow` does not exist |
| 3 | `EscrowAlreadyReleased` | Milestone or escrow already released |
| 4 | `EscrowAlreadyResolved` | Dispute already resolved |
| 5 | `EscrowNotInDispute` | Operation requires dispute to be open |
| 6 | `EscrowOpenedForDisputeResolution` | Cannot release while disputed |
| 7 | `EscrowNotCompleted` | Milestones not fully approved |
| 8 | `EscrowBalanceNotEnoughToSendEarnings` | Contract token balance insufficient |
| 9 | `EscrowPropertiesMismatch` | `expected_escrow` doesn't match stored escrow |
| 10 | `FlagsMustBeFalse` | Reentrancy guard active, or milestone has pre-set flags |
| 11 | `AmountCannotBeZero` | Amount ≤ 0 |
| 12 | `PlatformFeeTooHigh` | Fee exceeds 99% cap |
| 13 | `InsufficientFundsForEscrowFunding` | Funder balance < amount |
| 14 | `TooManyEscrowsRequested` | Batch > 20 |
| 15 | `InsufficientFundsForResolution` | Balance < distribution total |
| 16 | `DistributionsMustEqualEscrowBalance` | Distribution total must exactly match the required total (disputed milestones' amounts in `resolve_dispute`, contract balance in `withdraw_remaining_funds`) |
| 17 | `AmountsToBeTransferredShouldBePositive` | Distribution entry ≤ 0 |
| 18 | `TotalAmountCannotBeZero` | Zero total in distribution |
| 19 | `TooManyDistributions` | > 50 distribution entries |
| 20 | `EscrowNotFullyProcessed` | `withdraw_remaining_funds` requires all milestones processed |
| 21 | `Overflow` | Integer overflow |
| 22 | `Underflow` | Integer underflow |
| 23 | `DivisionError` | Division by zero |
| 24 | `OnlyReleaseSignerCanReleaseEarnings` | |
| 25 | `OnlyDisputeResolverCanExecuteThisFunction` | |
| 26 | `UnauthorizedToChangeDisputeFlag` | Caller cannot open a dispute |
| 27 | `DisputeResolverCannotDisputeTheEscrow` | Resolver tried to open dispute |
| 28 | `OnlyAdminAddressExecuteThisFunction` | |
| 29 | `AdminAddressCannotBeChanged` | |
| 30 | `AdminAddressOverlapsWithOtherRole` | |
| 31–34 | Role list empty errors | `ApproversListEmpty` etc. |
| 35 | `NoMilestoneDefined` | |
| 36 | `TooManyMilestones` | > 50 |
| 37 | `TargetCannotBeZero` | |
| 38 | `PlatformAddressCannotBeChanged` | |
| 39 | `ReleaseMilestonesEmpty` | |
| 40 | `MilestoneAlreadyReleased` | |
| 41 | `InvalidMilestoneIndex` | |
| 42 | `BatchMilestoneDisputeEmpty` | |
| 43 | `MilestoneAlreadyDisputed` | |
| 44 | `RoleLimitExceeded` | > 5 per role |
| 45 | `DuplicateAddressInRole` | |
| 46 | `DisputeResolverOverlapsWithOtherRole` | |
| 47 | `MilestoneUpdateNotAllowedWithFunds` | Cannot change amount while funded |
| 48 | `TargetExceedsApprovers` | `target > approvers.len()` |
| 49 | `StringTooLong` | |
| 50 | `SignerMustBeApproverAndReleaseSigner` | For `approve_and_release_milestones` |

### `ReleaseError` (codes 1–14)
Parallel error type used by `release_funds` / `release_funds_inner` specifically. Mapped to `EscrowError` via `From<ReleaseError>`. Code 14 is `BatchTooLarge`: the `milestone_indices` batch exceeds the milestone count (maps to `TooManyMilestones`).

### `MilestoneError` (codes 1–15)
Used by milestone operations. Mapped to `EscrowError` via `From<MilestoneError>`.

---

## 11. Storage Layout

| Key | Type | TTL | Notes |
|-----|------|-----|-------|
| `DataKey::Admin` | `Address` | 1 year, set at deploy | Removed after `initialize_escrow` |
| `DataKey::ApprovedWasmHash` | `BytesN<32>` | 1 year, set at deploy | Removed after `initialize_escrow` |
| `DataKey::Escrow` | `Escrow` | 1 year, extended on every write | Main escrow state |
| `DataKey::FundedAmount` | `i128` | 1 year, extended on each fund | Running total of deposited tokens |
| `DataKey::Reentrancy` | `bool` | Temporary | Set before external calls in `withdraw_remaining_funds` and `resolve_dispute`, removed after |

All storage uses **persistent** storage (survives ledger closings, subject to TTL expiry).

TTL is always extended with `threshold = 17,280` (approximately 1 day) and `extend_to = 31,536,000` (approximately 1 year at ~2s per ledger).

---

## 12. Lifecycle Flow

```
DEPLOY (constructor)
  │  stores: Admin, ApprovedWasmHash
  ▼
INITIALIZE (initialize_escrow)
  │  admin.require_auth()
  │  stores: Escrow (with milestones)
  │  removes: Admin, ApprovedWasmHash
  ▼
FUND (fund_escrow) ← can be called multiple times
  │  signer transfers tokens to contract
  │  increments FundedAmount
  ▼
SERVICE PROVIDER WORKS
  │  change_milestone_status → updates status/evidence
  ▼
APPROVALS (approve_milestones)
  │  approvers vote per-milestone
  │  each milestone tracks approved_by + approval_count
  │  milestone is "approved" when approval_count >= target
  ▼
  ┌────────────────────────────────────────────────────┐
  │  HAPPY PATH                DISPUTE PATH            │
  │                                                    │
  │  release_funds             dispute_milestones      │
  │  (or approve_and_release)  sets is_disputed=true   │
  │                                                    │
  │  For each released MS:       resolve_dispute       │
  │  → TW fee → platform fee     sets resolved=true    │
  │  → net_amount → receiver     distributes fees      │
  │                                                    │
  │                            (if leftover funds)     │
  │                            withdraw_remaining_funds│
  └────────────────────────────────────────────────────┘
```

---

## 13. Security Properties

1. **Reentrancy protection** — `withdraw_remaining_funds` uses `DataKey::Reentrancy` flag (set before calls, removed after). Prevents reentrant calls via malicious token contracts.

2. **Effects before interactions** — `release_funds_execute` marks milestones as `released = true` and writes to storage *before* any token transfers.

3. **Expected-escrow check in `fund_escrow`** — The caller must supply the exact current escrow state as `expected_escrow`. If another transaction modified the escrow between the caller's read and this call, `EscrowPropertiesMismatch` is returned. This is a TOCTOU protection.

4. **Role isolation** — `dispute_resolvers` cannot overlap with `approvers`, `service_providers`, or `release_signers`. Prevents a resolver from both disputing and resolving.

5. **Admin cannot be operational** — `admin` is blocked from `approvers`, `service_providers`, `release_signers`, and `dispute_resolvers`. Admin is purely a configuration role.

6. **Immutable admin and platform** — Once set at initialization, neither `admin` nor `platform` address can be changed. Prevents privilege escalation post-deployment.

7. **Milestone flags must be clean at init** — All `approved_by`, `approval_count`, `released`, `is_disputed`, `resolved` must be zero/false when the escrow is initialized.

8. **Fee cap** — Total fees (`platform_fee + 0.30%`) cannot exceed 100%. Receiver always gets > 0 when amount > 0.

9. **Overflow-safe math** — All arithmetic goes through `BasicMath`/`SafeMath` with checked operations. Returns explicit error codes instead of panicking.

10. **WASM hash verification** — Factory deployment validates the WASM hash against the approved hash stored at construction time, preventing deployment of unapproved contract code.

---

## 14. Differences vs Single-Release

| Aspect | Multi-Release (`feat/multi-release-v2`) | Single-Release (`feat/single-release-v2`) |
|--------|-----------------------------------------|------------------------------------------|
| Escrow `amount` | Per-milestone (`Milestone.amount`) | Single field on `Escrow.amount` |
| Receiver | Per-milestone (`Milestone.receiver`) | Single `Roles.receiver` |
| Release | Per milestone index(es) | Entire escrow at once |
| Dispute scope | Per milestone | Entire escrow |
| `Dispute` struct location | Inside each `Milestone` | Inside `Escrow` |
| `released` flag location | Inside each `Milestone` | Inside `Escrow` |
| `MilestoneUpdate.new_amount` | Yes (can update amount) | No (amount is on escrow) |
| Deploy function name | `tw_new_multi_release_escrow` | `tw_new_single_release_escrow` |
| `dispute_milestones` signature | Takes `milestone_indices: Vec<u32>` | `dispute_escrow` — no indices |
| `resolve_dispute` signature | Takes `milestone_indices: Vec<u32>` | No indices (resolves whole escrow) |
| `InitEsc` event fields | `milestone_count`, `total_amount` | `amount`, `platform_fee`, `trustline`, `receiver` |
| `ReleaseEsc` event fields | `payouts: Vec<MilestonePayout>` | `receiver`, `amount`, `platform_fee`, `trustless_work_fee`, `net_amount` |
| `EscrowPropertyChanges` fields | No amount fields (amounts are per-milestone) | Includes `amount` flag + `old_amount`/`new_amount` (escrow-level amount) |
| `MilestoneAddedEntry` fields | `index`, `amount`, `description_hash` | `index`, `description_hash` (no per-milestone amount) |
| `MilestoneUpdatedEntry` fields | `index`, `new_amount`, `new_description_hash` | `index`, `new_description_hash` |
| `Roles.receiver` field | Not present | Present |
| Admin overlap check includes receiver | No | Yes (`admin != receiver`) |
| Dispute resolver overlap check includes receiver | No | Yes (`resolver != receiver`) |
