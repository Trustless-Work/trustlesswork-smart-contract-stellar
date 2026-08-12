# Trustless Work — Single-Release Escrow Contract Documentation

> **Branch:** `single-release-develop-v2`  
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
   - 5.2 [tw_new_single_release_escrow](#52-tw_new_single_release_escrow)
   - 5.3 [initialize_escrow](#53-initialize_escrow)
   - 5.4 [fund_escrow](#54-fund_escrow)
   - 5.5 [release_funds](#55-release_funds)
   - 5.6 [update_escrow](#56-update_escrow)
   - 5.7 [manage_milestones](#57-manage_milestones)
   - 5.8 [change_milestone_status](#58-change_milestone_status)
   - 5.9 [approve_milestones](#59-approve_milestones)
   - 5.10 [approve_and_release_milestones](#510-approve_and_release_milestones)
   - 5.11 [dispute_escrow](#511-dispute_escrow)
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
14. [Differences vs Multi-Release](#14-differences-vs-multi-release)

---

## 1. Overview

This is a **single-release escrow** smart contract deployed on the **Stellar Soroban** blockchain. It holds stablecoins (e.g. USDC) on behalf of parties and releases **the entire escrow amount in a single atomic transaction** when all milestones are approved.

Key characteristics:
- **One amount, one receiver** — the escrow has a single `amount` field and a single `receiver` in `Roles`
- **Milestones are tracking-only** — milestones track status and approvals but do not carry individual amounts or receivers. They serve as gates: ALL milestones must be approved before any funds can be released
- **Single dispute state** — the entire escrow can be disputed or resolved as a unit
- **Atomic release** — `release_funds` releases the full escrow amount in one call

The contract uses a **factory + instance** deployment pattern: a parent contract deploys child escrow contracts via `tw_new_single_release_escrow`. Each deployed child holds exactly one escrow.

---

## 2. Architecture & File Structure

```
contracts/escrow/
├── Cargo.toml                       # Package: soroban-sdk 26.0.0
└── src/
    ├── lib.rs                       # Module tree root, no_std
    ├── contract.rs                  # EscrowContract — all public entry points
    ├── error.rs                     # EscrowError, MilestoneError enums
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

All types are decorated with `#[contracttype]` for ABI serialization on Soroban.

### `Escrow`
The root storage struct, stored under `DataKey::Escrow`.

```rust
pub struct Escrow {
    pub engagement_id: String,       // Unique ID (max 100 chars)
    pub title: String,               // Human-readable title (max 100 chars)
    pub roles: Roles,                // All role assignments (includes receiver)
    pub description: String,         // Description (max 500 chars)
    pub amount: i128,                // Total token amount held in escrow
    pub platform_fee: u32,           // Platform fee in basis points (e.g. 300 = 3%)
    pub milestones: Vec<Milestone>,  // Ordered list of tracking milestones (max 50)
    pub dispute: Dispute,            // Single dispute state for the whole escrow
    pub released: bool,              // True once funds have been released
    pub trustline: Trustline,        // Token contract address
    pub receiver_memo: u32,          // Optional memo for receiver identification
}
```

> **Critical difference from multi-release:** `amount`, `dispute`, and `released` live on the `Escrow` struct — not on individual milestones.

### `Milestone`
Milestones are **tracking units only** — they track work status and approvals but have no funds or receiver of their own.

```rust
pub struct Milestone {
    pub description: String,            // What work this milestone covers (max 500 chars)
    pub status: String,                 // Free-text status string (max 50 chars)
    pub evidence: String,               // URL or proof of completion (max 500 chars)
    pub approvals: MilestoneApprovals,  // Threshold approval tracking
}
```

> Milestones do **not** have `amount`, `receiver`, `released`, or `dispute` fields. These are all on the `Escrow` itself.

### `MilestoneApprovals`
Threshold-based approval system per milestone.

```rust
pub struct MilestoneApprovals {
    pub target: u32,              // Approvals required (must be > 0, ≤ approvers.len())
    pub approval_count: u32,      // Count of unique approvals received so far
    pub approved_by: Vec<Address>, // Addresses that have already voted
}
```

A milestone is considered **approved** when `approval_count >= target`. For release, **all** milestones must be approved.

### `Dispute`
Single dispute state for the entire escrow.

```rust
pub struct Dispute {
    pub is_disputed: bool,  // True while dispute is open and unresolved
    pub reason: String,     // Reason text provided when dispute was opened (max 500 chars)
    pub resolved: bool,     // True once dispute has been resolved (terminal state)
}
```

### `Roles`
All actor addresses for the escrow. Includes `receiver` directly.

```rust
pub struct Roles {
    pub approvers: Vec<Address>,         // Can approve milestones (max 5)
    pub service_providers: Vec<Address>, // Can change milestone status (max 5)
    pub platform: Address,               // Receives platform fee on release
    pub release_signers: Vec<Address>,   // Can trigger fund release (max 5)
    pub dispute_resolvers: Vec<Address>, // Can resolve disputes (max 5)
    pub receiver: Address,               // Single recipient of the net release amount
    pub admin: Address,                  // Can update escrow and manage milestones
    pub observers: Vec<Address>,         // Read-only observers (max 5)
}
```

> **Key difference from multi-release:** `Roles.receiver` is a single address — there is exactly one recipient for the entire escrow.

### `Trustline`
```rust
pub struct Trustline {
    pub address: Address, // Stellar asset contract (e.g. USDC SEP-41 token)
}
```

### `MilestoneStatusUpdate`
Input for batch status changes.

```rust
pub struct MilestoneStatusUpdate {
    pub milestone_index: u32,
    pub new_status: String,
    pub new_evidence: Option<String>,
}
```

### `MilestoneUpdate`
Input for `manage_milestones` to modify existing milestones.

```rust
pub struct MilestoneUpdate {
    pub index: u32,
    pub new_description: Option<String>,
    // Note: no new_amount field — amount lives on the Escrow, not milestones
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
    Admin,            // Temporary admin used during initialization
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
| `approvers` | Client(s) or designated reviewers | `approve_milestones`, `approve_and_release_milestones`, `dispute_escrow` |
| `service_providers` | Freelancer/contractor side | `change_milestone_status`, `dispute_escrow` |
| `release_signers` | Platform or designated signer | `release_funds`, `approve_and_release_milestones` |
| `dispute_resolvers` | Neutral arbitrator | `resolve_dispute`, `withdraw_remaining_funds` |
| `platform` | Fee receiver | Receives `platform_fee` on every release |
| `receiver` | The payee | Receives `net_amount` when escrow is released; can `dispute_escrow` |
| `observers` | Auditors/watchers | No write access; for off-chain indexing |

**Role constraints enforced at initialization:**
- `admin` cannot overlap with `approvers`, `service_providers`, `release_signers`, `dispute_resolvers`, or `receiver`
- `dispute_resolvers` cannot overlap with `approvers`, `service_providers`, `release_signers`, or `receiver`
- No duplicate addresses within any role list
- Each role list is capped at **5 members** maximum
- `admin` and `platform` addresses **cannot be changed** after initialization (immutable)

---

## 5. Contract Entry Points (Public Functions)

### 5.1 Constructor
```rust
pub fn __constructor(e: Env, admin: Address, approved_wasm_hash: BytesN<32>)
```
Called once at contract deployment. Stores `admin` and `approved_wasm_hash` in persistent storage, TTL extended to ~1 year. Both keys are removed from storage after `initialize_escrow` completes.

### 5.2 `tw_new_single_release_escrow`
```rust
pub fn tw_new_single_release_escrow(
    env: Env,
    signer: Address,
    wasm_hash: BytesN<32>,
    salt: BytesN<32>,
    init_fn: Symbol,
    init_args: Vec<Val>,
    constructor_args: Vec<Val>,
) -> Result<(Address, Val), EscrowError>
```
**Factory deployment function.** Deploys a new child escrow contract and invokes its init function atomically.

- Fails if the calling contract already has an escrow initialized
- Validates `wasm_hash` matches stored `approved_wasm_hash`
- Requires `signer.require_auth()`
- Deploys via `env.deployer().with_address(deployer, salt).deploy_v2(wasm_hash, constructor_args)`
- Returns `(deployed_address, init_return_value)`

### 5.3 `initialize_escrow`
```rust
pub fn initialize_escrow(e: &Env, escrow_properties: Escrow) -> Result<Escrow, EscrowError>
```
Sets up the escrow with its full configuration. Can only be called once.

- Requires `stored_admin.require_auth()` (address in `DataKey::Admin`)
- After success, removes `DataKey::Admin` and `DataKey::ApprovedWasmHash` permanently
- Extends `DataKey::Escrow` TTL to ~1 year
- Emits `InitEsc` event with `engagement_id`, `amount`, `platform_fee`, `trustline`, `receiver`

**Validation:** Full escrow conditions check (see [Section 8](#8-validation-rules)).

### 5.4 `fund_escrow`
```rust
pub fn fund_escrow(
    e: &Env,
    signer: Address,
    expected_escrow: Escrow,
    amount: i128,
) -> Result<(), EscrowError>
```
Transfers tokens from `signer` into the contract. The `expected_escrow` parameter is a **TOCTOU protection** — the caller must pass the exact current state.

- Validates `amount > 0`
- Validates `stored_escrow == expected_escrow` (mismatch → `EscrowPropertiesMismatch`)
- Validates `signer` balance ≥ `amount`
- Calls `token_client.transfer(signer, contract, amount)`
- Increments `DataKey::FundedAmount`
- Emits `FundEsc` event

### 5.5 `release_funds`
```rust
pub fn release_funds(
    e: &Env,
    release_signer: Address,
    trustless_work_address: Address,
) -> Result<(), EscrowError>
```
Releases the **entire escrow amount** to the single `receiver`. All milestones must be approved.

- Only callable by a `release_signer`
- Requires `escrow.released == false`
- Requires `escrow.dispute.resolved == false`
- Requires `escrow.dispute.is_disputed == false`
- Requires `escrow.milestones` not empty
- Requires ALL milestones to have `approval_count >= target`
- Sets `escrow.released = true` **before** token transfers (effects-before-interactions)
- Transfers: `trustless_work_fee` → `trustless_work_address`, `platform_fee` → `platform`, `net_amount` → `roles.receiver`
- Emits `ReleaseEsc` event with full fee breakdown

### 5.6 `update_escrow`
```rust
pub fn update_escrow(
    e: &Env,
    admin_address: Address,
    escrow_properties: Escrow,
) -> Result<Escrow, EscrowError>
```
Allows `admin` to update escrow metadata and role assignments.

- Only callable by current `escrow.roles.admin`
- **Cannot change** `admin` or `platform` address
- If contract has funds (balance > 0): cannot change any field except milestones are preserved anyway
- Cannot be called while `dispute.is_disputed == true`
- Preserves existing `milestones`, `dispute`, and `released` state (incoming values ignored)
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
Adds new milestones or updates descriptions of existing milestones.

- Only callable by `admin`
- `MilestoneUpdate` can only change `new_description` (no amount changes — amount is on `Escrow`)
- New milestones: must have `amount > 0` (but amount refers to the escrow, not the milestone), `approvals.target > 0`, `target <= approvers.len()`
- String limits enforced on inputs: new-milestone `description`/`evidence` ≤ 500 chars, `status` ≤ 50 chars, and `new_description` ≤ 500 chars (`StringTooLong`)
- Total milestone count cannot exceed 50
- Cannot be called if `dispute.is_disputed`, `released`, or `dispute.resolved`
- Emits `MilestonesManaged` event

### 5.8 `change_milestone_status`
```rust
pub fn change_milestone_status(
    e: Env,
    updates: Vec<MilestoneStatusUpdate>,
    service_provider: Address,
) -> Result<(), MilestoneError>
```
Allows service providers to update milestone status and evidence in a batch.

- Only callable by a `service_provider`
- Batch: 1–50 updates; status: 1–50 chars; evidence: 0–500 chars
- Emits `MilestoneStatusChanged` event

### 5.9 `approve_milestones`
```rust
pub fn approve_milestones(
    e: Env,
    milestone_indices: Vec<u32>,
    approver: Address,
) -> Result<(), MilestoneError>
```
Records an approver's vote on a set of milestones.

- Only callable by an address in `approvers`
- Each approver can vote once per milestone
- Cannot approve a milestone that already reached its threshold
- Batch: 1–50, no duplicates
- Appends to `approved_by` and increments `approval_count`
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
Atomic approve + release in one transaction.

- `signer` must be in BOTH `approvers` AND `release_signers`
- Approves the specified milestone indices, then releases the entire escrow
- Emits both `MilestonesApproved` and `ReleaseEsc` events

> **Important:** This approves only the listed milestone indices but then checks that ALL milestones are approved before releasing. If other milestones are still unapproved the release will fail.

### 5.11 `dispute_escrow`
```rust
pub fn dispute_escrow(e: Env, signer: Address, reason: String) -> Result<(), EscrowError>
```
Opens a dispute on the **entire escrow** (not individual milestones).

- Authorized callers: `approvers`, `service_providers`, `platform`, `release_signers`, or `roles.receiver`
- `dispute_resolvers` are explicitly blocked from opening disputes
- Cannot dispute once the escrow is released (`EscrowAlreadyReleased`)
- Cannot dispute if already disputed (`EscrowAlreadyInDispute`) or resolved
- Reason: max 500 chars
- Sets `escrow.dispute.is_disputed = true`
- Emits `EscrowDisputed` event

### 5.12 `resolve_dispute`
```rust
pub fn resolve_dispute(
    e: Env,
    dispute_resolver: Address,
    trustless_work_address: Address,
    distributions: Map<Address, i128>,
) -> Result<(), EscrowError>
```
Resolves the dispute by distributing the **entire contract balance** among specified recipients.

- Only callable by a `dispute_resolver`
- Protected by a reentrancy guard (`DataKey::Reentrancy`; re-entry fails with `Reentrancy`)
- `distributions` sum must **equal exactly** the current contract token balance (`DistributionsMustEqualEscrowBalance`)
- Sets `dispute.resolved = true`, `dispute.is_disputed = false`
- Distributes after fee deduction proportionally
- Emits `DisputeResolved` event

> **Key difference from multi-release:** The distribution total must equal the full contract balance (not just a portion). This ensures all funds are accounted for.

### 5.13 `withdraw_remaining_funds`
```rust
pub fn withdraw_remaining_funds(
    e: Env,
    dispute_resolver: Address,
    trustless_work_address: Address,
    distributions: Map<Address, i128>,
) -> Result<(), EscrowError>
```
Handles leftover funds after the escrow has been either released or dispute-resolved.

- Protected by reentrancy guard (`DataKey::Reentrancy`)
- `all_processed = escrow.released || escrow.dispute.resolved`
- Requires the escrow to be disputed, dispute-resolved, **or released** — a released escrow lets the resolver sweep surplus funds without any dispute (`EscrowNotInDispute` otherwise)
- `distributions` sum must **exactly equal** the contract balance (`DistributionsMustEqualEscrowBalance`)
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
Cross-contract read. Calls `get_escrow` on another contract address.

### 5.16 `get_multiple_escrow_balances`
```rust
pub fn get_multiple_escrow_balances(
    e: &Env,
    addresses: Vec<Address>,
) -> Result<Vec<AddressBalance>, EscrowError>
```
Batch balance query. Returns token balance held by each contract address.

- Maximum 20 addresses

### 5.17 `extend_contract_ttl`
```rust
pub fn extend_contract_ttl(
    e: &Env,
    admin: Address,
    ledgers_to_extend: u32,
) -> Result<(), EscrowError>
```
Extends `DataKey::Escrow` TTL to prevent on-chain expiry.

- Only `admin`
- Min threshold: 17,280 ledgers (~1 day)
- Emits `TtlExtended` event

---

## 6. Core Managers

### 6.1 EscrowManager

Located in `src/core/escrow.rs`.

- `get_receiver(escrow)` → `Address` — reads from `escrow.roles.receiver` (not from milestone)
- `release_funds_execute(e, trustless_work_address, escrow)` — sets `escrow.released = true` first, then performs token transfers. Uses `escrow.amount` (not per-milestone)
- `change_escrow_properties` — preserves `milestones`, `dispute`, `released` from the existing escrow when updating
- Single release path: no `milestone_indices` — always releases the entire `escrow.amount`

### 6.2 MilestoneManager

Located in `src/core/milestone.rs`. Functionally identical to multi-release except milestones have no `amount`/`receiver`/`dispute`/`released` fields.

- `approve_milestones_inner` skips `require_auth` for use by `approve_and_release_milestones`
- `change_milestone_status` validates evidence length ≤ 500 chars (not 500 as in multi-release — same limit, different constant name)

### 6.3 DisputeManager

Located in `src/core/dispute.rs`.

- `dispute_escrow(e, signer, reason)` — disputes the whole escrow, no milestone indices
- `resolve_dispute(e, resolver, trustless_work, distributions)` — no `milestone_indices` parameter; validates `total == current_balance`
- `withdraw_remaining_funds` — checks `escrow.released || escrow.dispute.resolved` (single boolean, not per-milestone iteration)
- Uses `EscrowError::Reentrancy` (code 47) instead of `EscrowError::FlagsMustBeFalse` (code 10) for reentrancy guard

---

## 7. Fee System

Identical formula to multi-release:

**Constants:**
```
TRUSTLESS_WORK_FEE_BPS = 30   // 0.30%
BASIS_POINTS_DENOMINATOR = 10_000
```

**Formula on release:**
```
trustless_work_fee = escrow.amount * 30 / 10_000
platform_fee = escrow.amount * platform_fee_bps / 10_000
receiver_amount = escrow.amount - trustless_work_fee - platform_fee
```

**For dispute resolution:**
- Distribution total must equal the **full contract balance** (not just a portion)
- Fees are deducted from the total and remainder distributed proportionally
- Rounding dust assigned to last recipient

All arithmetic uses checked operations (`BasicMath`/`SafeMath`).

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
| Milestone description/status/evidence too long | `StringTooLong` |
| `platform_fee` > 9,900 bps | `PlatformFeeTooHigh` |
| `platform_fee + 30 > 10,000` | `PlatformFeeTooHigh` |
| `approvers` empty | `ApproversListEmpty` |
| `service_providers` empty | `ServiceProvidersListEmpty` |
| `release_signers` empty | `ReleaseSignersListEmpty` |
| `dispute_resolvers` empty | `DisputeResolversListEmpty` |
| Any role list > 5 members | `RoleLimitExceeded` |
| Duplicate address within a role list | `DuplicateAddressInRole` |
| `dispute_resolver` overlaps `approvers`/`service_providers`/`release_signers`/`receiver` | `DisputeResolverOverlapsWithOtherRole` |
| `admin` overlaps any role or `receiver` | `AdminAddressOverlapsWithOtherRole` |
| Milestones > 50 | `TooManyMilestones` |
| Milestone `approvals.target == 0` | `TargetCannotBeZero` |
| Milestone `target > approvers.len()` | `TargetExceedsApprovers` |
| Milestone has non-zero approval count/flags | `FlagsMustBeFalse` |

> Note: Milestone `amount` is not validated here because `Milestone` has no `amount` field. The escrow-level `amount` field is not validated to be > 0 at initialization (it can be 0; value flows in via `fund_escrow`).

### Release
| Rule | Error |
|------|-------|
| `escrow.released == true` | `EscrowAlreadyReleased` |
| `escrow.dispute.resolved == true` | `EscrowAlreadyResolved` |
| Caller not in `release_signers` | `OnlyReleaseSignerCanReleaseEarnings` |
| No milestones defined | `NoMilestoneDefined` |
| Not ALL milestones approved | `EscrowNotCompleted` |
| `escrow.dispute.is_disputed == true` | `EscrowOpenedForDisputeResolution` |
| Contract balance < `escrow.amount` | `EscrowBalanceNotEnoughToSendEarnings` |

### Dispute Resolution
| Rule | Error |
|------|-------|
| Caller not in `dispute_resolvers` | `OnlyDisputeResolverCanExecuteThisFunction` |
| `dispute.is_disputed == false` | `EscrowNotInDispute` |
| `distributions` total ≠ contract balance | `DistributionsMustEqualEscrowBalance` |
| Contract balance < total | `InsufficientFundsForResolution` |
| Total ≤ 0 | `TotalAmountCannotBeZero` |
| > 50 distribution entries | `TooManyDistributions` |

---

## 9. Events

| Event Struct | Topic | Fields |
|---|---|---|
| `InitEsc` | `tw_init`, `engagement_id` | `amount`, `platform_fee`, `trustline`, `receiver` |
| `FundEsc` | `tw_fund`, `engagement_id` | `funder`, `amount`, `funded_total` |
| `ReleaseEsc` | `tw_release`, `engagement_id` | `release_signer`, `receiver`, `amount`, `platform_fee`, `trustless_work_fee`, `net_amount` |
| `EscrowUpdated` | `tw_update`, `engagement_id` | `admin` |
| `MilestoneStatusChanged` | `tw_ms_change`, `engagement_id` | `service_provider`, `updates: Vec<MilestoneStatusEntry>` |
| `MilestonesApproved` | `tw_ms_approve`, `engagement_id` | `approver`, `milestone_indices` |
| `EscrowDisputed` | `tw_dispute`, `engagement_id` | `signer`, `reason` |
| `DisputeResolved` | `tw_disp_resolve`, `engagement_id` | `dispute_resolver`, `platform_fee`, `trustless_work_fee`, `distributions` |
| `FundsWithdrawn` | `tw_withdraw`, `engagement_id` | `dispute_resolver`, `platform_fee`, `trustless_work_fee`, `distributions` |
| `MilestonesManaged` | `tw_ms_manage`, `engagement_id` | `admin`, `added_count`, `updated_count` |
| `TtlExtended` | `tw_ttl_extend`, `engagement_id` | `admin`, `ledgers_to_extend` |

> `ReleaseEsc` emits flat fee fields directly (not a `Vec<MilestonePayout>` like multi-release).  
> `EscrowDisputed` has no `milestone_indices` (whole-escrow dispute).  
> `DisputeResolved` has no `milestone_indices` field.

---

## 10. Error Codes

### `EscrowError`
| Code | Name | Meaning |
|------|------|---------|
| 1 | `EscrowAlreadyInitialized` | Contract already initialized |
| 2 | `EscrowNotFound` | `DataKey::Escrow` not in storage |
| 3 | `EscrowAlreadyReleased` | Escrow already released |
| 4 | `EscrowAlreadyResolved` | Dispute already resolved |
| 5 | `EscrowAlreadyInDispute` | Cannot dispute an already-disputed escrow |
| 6 | `EscrowNotInDispute` | Operation requires active dispute |
| 7 | `EscrowOpenedForDisputeResolution` | Cannot release while disputed |
| 8 | `EscrowNotCompleted` | Not all milestones approved |
| 9 | `EscrowBalanceNotEnoughToSendEarnings` | Contract balance < escrow amount |
| 10 | `EscrowPropertiesMismatch` | expected_escrow ≠ stored_escrow |
| 11 | `FlagsMustBeFalse` | Milestone has pre-set approval/dispute flags |
| 12 | `AmountCannotBeZero` | Amount ≤ 0 |
| 13 | `PlatformFeeTooHigh` | Fee > 99% cap |
| 14 | `InsufficientFundsForEscrowFunding` | Funder balance < amount |
| 15 | `TooManyEscrowsRequested` | Batch > 20 |
| 16 | `InsufficientFundsForResolution` | Balance < distribution total |
| 17 | `DistributionsMustEqualEscrowBalance` | Distributions ≠ full balance |
| 18 | `AmountsToBeTransferredShouldBePositive` | Entry ≤ 0 |
| 19 | `TotalAmountCannotBeZero` | Zero total |
| 20 | `TooManyDistributions` | > 50 entries |
| 21 | `EscrowNotFullyProcessed` | Withdraw requires released or resolved |
| 22 | `Overflow` | Integer overflow |
| 23 | `Underflow` | Integer underflow |
| 24 | `DivisionError` | Division by zero |
| 25 | `OnlyReleaseSignerCanReleaseEarnings` | |
| 26 | `OnlyDisputeResolverCanExecuteThisFunction` | |
| 27 | `UnauthorizedToChangeDisputeFlag` | Cannot open a dispute |
| 28 | `DisputeResolverCannotDisputeTheEscrow` | Resolver tried to open dispute |
| 29 | `OnlyAdminAddressExecuteThisFunction` | |
| 30 | `AdminAddressCannotBeChanged` | |
| 31 | `AdminAddressOverlapsWithOtherRole` | |
| 32–35 | Role list empty errors | `ApproversListEmpty` etc. |
| 36 | `NoMilestoneDefined` | |
| 37 | `TooManyMilestones` | |
| 38 | `TargetCannotBeZero` | |
| 39 | `PlatformAddressCannotBeChanged` | |
| 40 | `InvalidMilestoneIndex` | |
| 41 | `RoleLimitExceeded` | > 5 per role |
| 42 | `DuplicateAddressInRole` | |
| 43 | `DisputeResolverOverlapsWithOtherRole` | |
| 44 | `MilestoneUpdateNotAllowedWithFunds` | Cannot update milestones while funded |
| 45 | `TargetExceedsApprovers` | target > approvers.len() |
| 46 | `StringTooLong` | |
| 47 | `Reentrancy` | Reentrancy guard active |
| 48 | `SignerMustBeApproverAndReleaseSigner` | For `approve_and_release_milestones` |

> **Numbering difference from multi-release:** Single-release adds `EscrowAlreadyInDispute` at code 5, shifting all subsequent codes. Single-release also uses `Reentrancy` (47) where multi-release uses `FlagsMustBeFalse` (10) for the reentrancy guard.

### `MilestoneError` (codes 1–15)
Same as multi-release. Mapped to `EscrowError` via `From<MilestoneError>`.

---

## 11. Storage Layout

| Key | Type | TTL | Notes |
|-----|------|-----|-------|
| `DataKey::Admin` | `Address` | 1 year, set at deploy | Removed after `initialize_escrow` |
| `DataKey::ApprovedWasmHash` | `BytesN<32>` | 1 year, set at deploy | Removed after `initialize_escrow` |
| `DataKey::Escrow` | `Escrow` | 1 year, extended on every write | Main escrow state (includes `amount`, `dispute`, `released`) |
| `DataKey::FundedAmount` | `i128` | 1 year, extended on each fund | Running total of deposited tokens |
| `DataKey::Reentrancy` | `bool` | Temporary | Set before external calls in `withdraw_remaining_funds`, removed after |

TTL threshold = 17,280 ledgers; extend_to = 31,536,000 ledgers (~1 year at ~2s/ledger).

---

## 12. Lifecycle Flow

```
DEPLOY (constructor)
  │  stores: Admin, ApprovedWasmHash
  ▼
INITIALIZE (initialize_escrow)
  │  admin.require_auth()
  │  escrow.amount set here
  │  stores: Escrow (milestones are tracking-only)
  │  removes: Admin, ApprovedWasmHash
  ▼
FUND (fund_escrow) ← can be called multiple times
  │  signer transfers tokens to contract
  │  increments FundedAmount
  ▼
SERVICE PROVIDER WORKS
  │  change_milestone_status → updates status/evidence on each milestone
  ▼
APPROVALS (approve_milestones)
  │  approvers vote per-milestone
  │  ALL milestones must reach their threshold before release
  ▼
  ┌──────────────────────────────────────────────────────┐
  │  HAPPY PATH                  DISPUTE PATH            │
  │                                                      │
  │  release_funds               dispute_escrow(reason)  │
  │  (or approve_and_release)    sets dispute.is_disputed │
  │                                                      │
  │  ALL milestones approved?      resolve_dispute       │
  │  → TW fee → platform fee       distributions must    │
  │  → net_amount → receiver       equal full balance    │
  │  sets released = true          sets dispute.resolved  │
  │                                                      │
  │                              (if leftover funds)     │
  │                              withdraw_remaining_funds │
  └──────────────────────────────────────────────────────┘
```

---

## 13. Security Properties

1. **Reentrancy protection** — `withdraw_remaining_funds` uses `DataKey::Reentrancy`. Different error code than multi-release: `EscrowError::Reentrancy` (47) vs multi-release's `FlagsMustBeFalse` (10).

2. **Effects before interactions** — `release_funds_execute` sets `escrow.released = true` and persists to storage *before* any token transfers.

3. **Expected-escrow check in `fund_escrow`** — Caller must supply exact current escrow state. Prevents TOCTOU issues.

4. **Full-balance resolution** — `resolve_dispute` requires `distributions` to equal the exact contract balance, ensuring no funds are stranded.

5. **Role isolation** — `dispute_resolvers` cannot overlap with `approvers`, `service_providers`, `release_signers`, or `receiver`. Prevents a resolver from simultaneously being a party to the dispute.

6. **Admin cannot be operational** — `admin` is blocked from all operational roles including `receiver`. Admin is purely a configuration role.

7. **Immutable admin and platform** — Cannot be changed after initialization.

8. **ALL-or-nothing release** — Every defined milestone must be approved before `release_funds` succeeds. There is no partial release mechanism.

9. **Single dispute flag** — `EscrowAlreadyInDispute` (code 5) prevents opening a dispute twice. Multi-release has a similar check per-milestone with `MilestoneAlreadyDisputed`.

10. **Overflow-safe math** — All arithmetic uses checked operations with explicit error codes.

11. **WASM hash verification** — Factory deployment validates approved WASM hash, preventing unapproved code deployment.

---

## 14. Differences vs Multi-Release

| Aspect | Single-Release (`feat/single-release-v2`) | Multi-Release (`feat/multi-release-v2`) |
|--------|-----------------------------------------|------------------------------------------|
| Escrow `amount` | Single field on `Escrow.amount` | Per-milestone (`Milestone.amount`) |
| Receiver | Single `Roles.receiver` | Per-milestone (`Milestone.receiver`) |
| Release | Entire escrow in one call, no indices | Per milestone index(es) |
| Dispute scope | Entire escrow | Per milestone |
| `Dispute` struct location | Inside `Escrow` | Inside each `Milestone` |
| `released` flag location | Inside `Escrow` | Inside each `Milestone` |
| `MilestoneUpdate.new_amount` | Not present | Present |
| Deploy function name | `tw_new_single_release_escrow` | `tw_new_multi_release_escrow` |
| Dispute function name | `dispute_escrow(signer, reason)` | `dispute_milestones(signer, indices, reason)` |
| `resolve_dispute` signature | No `milestone_indices` | Takes `milestone_indices: Vec<u32>` |
| Distribution requirement | Must equal full contract balance | Must not exceed sum of disputed milestones |
| `InitEsc` event fields | `amount`, `platform_fee`, `trustline`, `receiver` | `milestone_count`, `total_amount` |
| `ReleaseEsc` event fields | Flat fields: `receiver`, `amount`, `platform_fee`, `trustless_work_fee`, `net_amount` | `payouts: Vec<MilestonePayout>` |
| `EscrowAlreadyInDispute` error | Code 5 (explicit) | Not present (checked per-milestone as `MilestoneAlreadyDisputed`) |
| Reentrancy error | `EscrowError::Reentrancy` (code 47) | `EscrowError::FlagsMustBeFalse` (code 10) |
| `Roles.receiver` field | Present | Not present |
| Admin overlap includes receiver | Yes | No |
| Dispute resolver overlap includes receiver | Yes | No |
| `all_processed` check in `withdraw` | `released \|\| dispute.resolved` | All milestones `released \|\| dispute.resolved` |
| Release requires milestones | All must be approved | Only specified indices must be approved |
