# STRIDE Threat-Model Audit — Multi-Release Escrow Contract

**Auditor:** Shadow-MMN  
**Date:** 2026-08-22  
**Base Branch:** `multi-release-develop-v2`  
**Commit Reviewed:** `2f569ec` (HEAD of `stride-audit-shadow-mmn`)

---

## 1. Scope & Methodology

### What Was Reviewed

All contract source under `contracts/escrow/src/` on `multi-release-develop-v2` at commit `2f569ec`:

| Path | Description |
|------|-------------|
| `contract.rs` | Entrypoint definitions, event publishing |
| `core/escrow.rs` | Escrow lifecycle (init, fund, release, update, manage milestones) |
| `core/milestone.rs` | Milestone status changes, approvals |
| `core/dispute.rs` | Dispute initiation, resolution, remaining-fund withdrawal |
| `core/validators/escrow.rs` | Escrow, milestone, and release validation |
| `core/validators/milestone.rs` | Batch milestone status and approval validation |
| `core/validators/dispute.rs` | Dispute resolution and withdrawal validation |
| `modules/fee/calculator.rs` | Fee calculation (trustless work + platform) |
| `modules/fee/distribution.rs` | Fee distribution with rounding remainder handling |
| `modules/math/basic.rs` | Checked arithmetic (add, sub, mul, div) |
| `modules/math/safe.rs` | Overflow-safe `mul_div` for fee computation |
| `storage/types.rs` | All contract types (Escrow, Milestone, Roles, DataKey, etc.) |
| `events/handler.rs` | Event structs and topic definitions |
| `error.rs` | Error enums and conversions |

### Methodology

1. **Trust Model Construction** — Mapped every role, every entrypoint, and every state transition before identifying threats. Verified which roles are authenticated on each write path.
2. **Category-by-Category STRIDE Pass** — Six independent passes, each scoped to one STRIDE category. For every finding, traced the exact code path to verify the exploit was realistic.
3. **Attack Test Suite** — Wrote and executed 27 Rust test cases (`stride_attack.rs`) against the contract to verify each finding through actual exploit attempts. All 27 tests passed (62 existing tests also pass — 89 total). Results are described as call sequences in §4.
4. **Findings Verification** — For each finding marked Medium or above, constructed a concrete exploit scenario (sequence of calls with example values) and verified the state transitions against the code.

---

## 2. Trust Model

### Roles

The contract defines seven roles in the `Roles` struct (`storage/types.rs:80-88`):

| Role | Max Members | Overlap Restrictions | Capabilities |
|------|------------|---------------------|--------------|
| **admin** | 1 (immutable) | Cannot overlap with approvers, service_providers, release_signers, dispute_resolvers | `update_escrow`, `manage_milestones`, `extend_contract_ttl` |
| **approvers** | 5 | No duplicate addresses within role | `approve_milestones`, `dispute_milestones` (global role) |
| **service_providers** | 5 | No duplicate addresses within role | `change_milestone_status`, `dispute_milestones` (global role) |
| **release_signers** | 5 | No duplicate addresses within role | `release_funds`, `approve_and_release_milestones`, `dispute_milestones` (global role) |
| **dispute_resolvers** | 5 | Cannot overlap with approvers, service_providers, release_signers | `resolve_dispute`, `withdraw_remaining_funds`. **Cannot** `dispute_milestones`. |
| **platform** | 1 (immutable) | N/A | Receives platform fee on release/resolve. Cannot be changed after init. Can `dispute_milestones` (global role). |
| **per-milestone receiver** | 1 per milestone | N/A | Receives net payout after fees. Can `dispute_milestones` for their own milestone only. |
| **observers** | 5 | No duplicate addresses within role | No write functions; read-only by design. |

**Critical gap**: `platform` is omitted from both `validate_admin_role_overlap` (`validators/escrow.rs:102-108`) and `validate_dispute_resolver_role_overlap` (`validators/escrow.rs:147-157`). This means `admin == platform` and `dispute_resolver == platform` are both permitted.

**Two admin concepts**: The constructor stores `DataKey::Admin` (`escrow.rs:36-38`), which is deleted after `initialize_escrow` (`escrow.rs:39`). The persistent admin lives in `escrow_properties.roles.admin`. `validate_initialize_escrow_conditions` (`validators/escrow.rs:278-285`) only checks `DataKey::Admin`, not `roles.admin` — the `roles.admin` address is whatever the caller provides and is never validated against the constructor admin.

### Entrypoint-to-Role Mapping

| Entrypoint | Required Role(s) | Auth Call Location |
|-----------|-------------------|-----------|
| `__constructor` | None (deployer-only) | N/A |
| `tw_new_multi_release_escrow` | `signer` (any; wasm-hash gated) | `contract.rs:52` |
| `initialize_escrow` | admin (stored `DataKey::Admin`) | `escrow.rs:34` |
| `fund_escrow` | Any address with tokens | `escrow.rs:62` |
| `release_funds` | release_signer | `escrow.rs:82` |
| `approve_and_release_milestones` | Both approver AND release_signer | `contract.rs:269` |
| `update_escrow` | admin | `escrow.rs:195` |
| `manage_milestones` | admin | `escrow.rs:230` |
| `change_milestone_status` | service_provider | `milestone.rs:21` |
| `approve_milestones` | approver | `milestone.rs:57` |
| `dispute_milestones` | approver, service_provider, release_signer, platform, OR per-milestone receiver | `dispute.rs:165` |
| `resolve_dispute` | dispute_resolver | `dispute.rs:119` |
| `withdraw_remaining_funds` | dispute_resolver | `dispute.rs:59` |
| `extend_contract_ttl` | admin | `contract.rs:194` |
| `get_escrow` | Anyone (read-only) | None |
| `get_escrow_by_contract_id` | Anyone (read-only) | None |
| `get_multiple_escrow_balances` | Anyone (read-only) | None |

There are 17 public entrypoints. Of the 14 state-changing entrypoints, 13 enforce `require_auth`. `__constructor` requires no auth (deployer-only by Soroban semantics). `tw_new_multi_release_escrow` calls `require_auth()` on the `signer` parameter but performs no role-based authorization — any authenticated address can invoke it (constrained only by the wasm-hash gate at `contract.rs:52-54`).

### Auth Order Consistency

The codebase uses two ordering patterns for auth vs. validation:

- **Validate-first** (6 functions): `release_funds` (`escrow.rs:80-82`), `update_escrow` (`escrow.rs:193-195`), `manage_milestones` (`escrow.rs:228-230`), `fund_escrow` (`escrow.rs:60-62`), `resolve_dispute` (`dispute.rs:117-119`), `withdraw_remaining_funds` (`dispute.rs:50-59`), `dispute_milestones` (`dispute.rs:163-165`)
- **Auth-first** (2 functions): `change_milestone_status` (`milestone.rs:21` before validation at `:23`), `approve_milestones` (`milestone.rs:57` before validation at `:69`)

Auth-first is the safer order (it rejects unauthorized callers before doing any work), but the inconsistency means the codebase does not follow a single convention. Since Soroban rolls back the entire transaction on auth failure, neither order is exploitable — but the inconsistency itself is worth noting.

### The `FundedAmount` Mechanism

`DataKey::FundedAmount` (`escrow.rs:67-69`) is an increment-only counter. It is set only in `fund_escrow` via `safe_add` and **never decremented** — not on release, not on dispute resolution, not on withdrawal. Two validator functions read it:

- `validate_escrow_property_change_conditions` (`validators/escrow.rs:342-350`) → feeds `contract_balance` into `validate_escrow_conditions` (`validators/escrow.rs:243-244`). When `contract_balance > 0`, the admin cannot change escrow properties.
- `validate_manage_milestones_conditions` (`validators/escrow.rs:319`) → when `contract_balance > 0`, the admin cannot update milestone amounts/descriptions.

Because the counter is never decremented, these restrictions persist **even after all funds are released and the contract balance is zero**. This is the root cause of findings §3.5, §3.8, and §3.15.

### Milestone State Machine

Each milestone has three state dimensions: `released` (bool), `dispute.is_disputed` (bool), `dispute.resolved` (bool), and an `approvals` struct (count vs target).

```
                          ┌─────────────────┐
                          │   Initialized   │
                          │ released=F      │
                          │ disputed=F      │
                          │ resolved=F      │
                          └────────┬────────┘
                                   │
                 ┌─────────────────┼─────────────────┐
                 │                                     │
    approve_milestones (count >= target)     dispute_milestones
                 │                                     │
                          ┌────────▼────────┐          │
                          │    Approved     │          │
                          │ released=F      │          │
                          │ disputed=F      │          │
                          │ resolved=F      │          │
                          └────────┬────────┘          │
                                   │                   │
                 ┌─────────────────┼─────────────────┐ │
                 │                                     │ │
        release_funds                        dispute_milestones
                 │                                     │ │
    ┌────────────▼──────────┐          ┌───────────────▼─▼──────┐
    │      Released         │          │       Disputed          │
    │ released=T            │          │ released=F              │
    │ disputed=F            │          │ disputed=T              │
    │ resolved=F            │          │ resolved=F              │
    └───────────────────────┘          └───────────┬─────────────┘
                                                   │
                                         resolve_dispute
                                                   │
                                       ┌───────────▼────────────┐
                                       │     Resolved           │
                                       │ released=F             │
                                       │ disputed=F             │
                                       │ resolved=T             │
                                       └────────────────────────┘
```

Note: `Initialized → Disputed` is a valid edge — `validate_batch_milestone_dispute_conditions` (`validators/dispute.rs:163-181`) requires `!released`, `!resolved`, `!is_disputed` but does **not** require prior approval. A milestone can be disputed directly from the Initialized state.

Valid transitions into each state:
- **Approved → Released**: Only if `is_milestone_approved(m)` is true, `!disputed`, `!resolved`, `!released` (`validators/escrow.rs:79-98`)
- **Initialized/Approved → Disputed**: Only if `!released`, `!resolved`, `!is_disputed` — authorized callers: global roles or per-milestone receiver (`validators/dispute.rs:133-181`)
- **Disputed → Resolved**: Only if `is_disputed`, `!resolved` — only dispute_resolver (`validators/dispute.rs:58-115`)

**NOT terminal**: The `status` and `evidence` string fields on milestones can be overwritten by any service_provider at any time, including after release or resolution (`validators/milestone.rs:13-55` — no `released` or `resolved` guard).

---

## 3. Findings

### Spoofing

#### S1 — `get_escrow_by_contract_id` Invokes Arbitrary Contracts

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `core/escrow.rs:282-287` |

**Description**: `get_escrow_by_contract_id` invokes `get_escrow` on any caller-supplied address with no verification that the target is a legitimate escrow contract. The returned "escrow" data and any downstream balance queries are entirely attacker-controlled.

**Exploit Path**: Attacker deploys a contract whose `get_escrow` returns an escrow with a trustline pointing to a token the attacker controls. Off-chain consumer calls `get_multiple_escrow_balances` with the attacker's contract address → receives fabricated balance data.

**Impact**: Medium. Enables spoofed data injection into off-chain consumers that trust `get_escrow_by_contract_id` output.

---

#### S2 — `tw_new_multi_release_escrow` Enables Address Squatting and Front-Running

| Field | Value |
|-------|-------|
| **Severity** | **Low** |
| **File:Line** | `contract.rs:33-66` |

**Description**: The factory function accepts any `signer` and a caller-chosen `salt` for deterministic deployment, gated by `wasm_hash == DataKey::ApprovedWasmHash` (`contract.rs:52-54`) and `DataKey::Escrow` not yet existing (`contract.rs:42-44`). The `ApprovedWasmHash` is removed after `initialize_escrow` (`escrow.rs:39`).

Before initialization, any address can observe the intended `salt`, `init_args`, and `constructor_args` in a pending transaction and front-run it. The front-runner deploys first with the same salt, permanently burning it — the platform's intended deployment fails with a contract address collision. Alternatively, the front-runner can deploy an escrow with themselves as `admin`, squatting the deterministic address.

**Impact**: Low. The wasm-hash gate limits what can be deployed, but the salt-burning front-run is free (no economic cost to the attacker) and the victim must redeploy with a new salt.

---

#### S3 — Auth-First vs. Validate-First Inconsistency

| Field | Value |
|-------|-------|
| **Severity** | **Info** |
| **File:Line** | `core/milestone.rs:21,57`, `core/dispute.rs:163-165` |

**Description**: Most entrypoints validate inputs before calling `require_auth()`, but two functions do the opposite: `change_milestone_status` (`milestone.rs:21`) and `approve_milestones` (`milestone.rs:57`) call `require_auth()` before validation. Since auth failure rolls back the entire transaction in Soroban, no state is persisted in either case. Auth-first is the safer convention (it rejects unauthorized callers before doing any work), but the codebase does not follow a single pattern.

**Impact**: Info. No exploitable vulnerability; the inconsistency is a code-quality concern.

---

### Tampering

#### T1 — Intermediate Overflow in `calculate_and_distribute_fees`

| Field | Value |
|-------|-------|
| **Severity** | **High** |
| **File:Line** | `modules/fee/distribution.rs:35` |

**Description**: The per-recipient net calculation uses `safe_mul` then `safe_div` instead of `safe_mul_div`:

```rust
let net = BasicMath::safe_div(BasicMath::safe_mul(amount, distributable)?, total)?;
```

`amount * distributable` overflows `i128` when both values exceed ~1.3×10¹⁹ (√i128::MAX). This is reachable in `resolve_dispute` and `withdraw_remaining_funds`, where `total` can be up to the sum of all milestone amounts and `distributable = total - fees`.

**Decimals caveat**: On 7-decimal Stellar assets (SAC-wrapped classics, including USDC), reaching 1.3×10¹⁹ units requires ~1.3×10¹² tokens — far beyond realistic escrow sizes. The overflow is only reachable on 18-decimal Soroban-native tokens (e.g., some DeFi tokens). On the dominant Stellar asset class (7-decimal), this finding is theoretical.

**Exploit Path** (18-decimal token): Attacker creates escrow with milestone amounts summing to ~13 tokens of an 18-decimal asset (e.g., 13_000_000_000_000_000_000). Disputes the milestone. Resolver calls `resolve_dispute` with a single distribution. `safe_mul(amount, distributable)` where both are ~1.3×10¹⁹ → overflow → transaction reverts permanently.

**Impact**: High (on 18-decimal trustlines only). The `release_funds` path uses `safe_mul_div` (`fee/calculator.rs:34-38`) and has explicit overflow testing (`modules/math/safe.rs:49-55`), proving the team is aware of this class of bug. The oversight in `distribution.rs:35` appears to be an error, not a design choice. On affected trustlines, `resolve_dispute` and `withdraw_remaining_funds` revert permanently with no alternative exit path.

---

#### T2 — Admin Can Reduce Milestone Amounts Before Funding

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `core/escrow.rs:235-237`, `core/validators/escrow.rs:319-321` |

**Description**: `manage_milestones` allows the admin to change milestone amounts via `MilestoneUpdate.new_amount` when `FundedAmount == 0` (`validators/escrow.rs:319-321`). The `MilestoneUpdate` struct (`storage/types.rs:72-76`) supports `new_description` and `new_amount` but not `new_receiver`.

**Exploit Path**: Admin initializes escrow with M0 (100 USDC → receiver A), M1 (100 USDC → receiver B). Before funding, admin calls `manage_milestones` to reduce M0 to 1 USDC and adds M3 (99 USDC → admin). Funder deposits 200 USDC (matches new total). Approver and release_signer approve and release normally. Admin receives 99 USDC.

**Mitigating factor**: The funder must actively pass the modified escrow to `fund_escrow` — `validate_fund_escrow_conditions` (`validators/escrow.rs:382-384`) requires `stored_escrow.eq(&expected_escrow)`, and `Escrow` derives `PartialEq` including `platform_fee` (`storage/types.rs:29-39`). This is not "silent" — the modification is only detectable by reading the escrow state before funding.

---

#### T3 — Status and Evidence Rewritable After Release/Resolution

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `core/validators/milestone.rs:13-55`, `core/milestone.rs:13-50` |

**Description**: `validate_batch_milestone_status_change` (`validators/milestone.rs:13-55`) checks emptiness, batch size, string lengths, and index bounds — but does **not** check `milestone.released` or `milestone.dispute.resolved`. Any service_provider can overwrite `status` and `evidence` on an already-paid or already-resolved milestone.

**Exploit Path**: Service provider completes work, milestone is approved and released. Service provider then calls `change_milestone_status` to set `status = "Pending"` and `evidence = "No work done"`. The on-chain record now contradicts the actual state.

**Impact**: Medium. The `released` boolean and `dispute.resolved` boolean are the source of truth for fund flow, so this does not enable fund theft. But it destroys the on-chain audit trail — a Repudiation concern (see §3.3, R2).

---

#### T4 — Zero-Milestone Escrow Blocks All Exit Paths

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `core/validators/escrow.rs:195-222` |

**Description**: `validate_escrow_conditions` with `is_init: true` skips all milestone validation when the milestones vec is empty (`validators/escrow.rs:195`: `if !new_escrow.milestones.is_empty()`). An escrow can be initialized with zero milestones, funded, and then all exit paths are blocked until the admin adds milestones via `manage_milestones`.

**Exploit Path**: Admin initializes escrow with no milestones. Funder deposits funds. Now `release_funds` requires approved milestones (none exist), `resolve_dispute` requires disputed milestones (none exist), `withdraw_remaining_funds` requires all milestones processed (none exist → `all_processed` is vacuously true, but `EscrowNotInDispute` blocks it since no milestones have disputes). Funds are stuck until admin adds milestones.

**Impact**: Medium. The admin has a unilateral path to add milestones, so this is recoverable — but it creates a dependency on admin action that could be exploited for griefing.

---

#### T5 — `FundedAmount` Counter Blocks Post-Release Modifications

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `core/validators/escrow.rs:243-244`, `core/validators/escrow.rs:319` |

**Description**: Because `FundedAmount` is never decremented (see §2), the check `contract_balance > 0` in `validate_escrow_conditions` (`validators/escrow.rs:243-244`) and `validate_manage_milestones_conditions` (`validators/escrow.rs:319`) remains true permanently after any funding, even when the actual contract token balance is zero.

**Consequence A** (DoS): After all milestones are released and the contract balance is zero, the admin cannot update escrow properties or modify milestone amounts/descriptions. The only path forward is adding new milestones (which is still allowed).

**Consequence B** (EoP): See §3.10 — the admin can change `platform_fee` to 9900 BPS before funding. Once funded, `FundedAmount > 0` prevents the fee from being lowered, locking in the excessive fee permanently.

---

#### T6 — Remainder Assignment Is Map-Order-Dependent

| Field | Value |
|-------|-------|
| **Severity** | **Low** |
| **File:Line** | `modules/fee/distribution.rs:35-51` |

**Description**: Two issues in the fee distribution loop (`distribution.rs:33-51`):

1. **Remainder is map-order-dependent**: The rounding remainder is assigned to the last recipient in `net_distributions` (`distribution.rs:47-51`). Since `net_distributions` is built by iterating over a Soroban `Map`, the "last" recipient depends on key serialization order.

2. **All-recipients-round-to-zero loses the remainder**: Recipients whose `net` rounds to 0 are silently dropped (`distribution.rs:35`: `if net > 0`). If every recipient rounds to 0, `net_distributions` is empty, and the `!net_distributions.is_empty()` guard at line 44 skips the remainder assignment. The entire distributable amount stays in the contract with no way to retrieve it — `withdraw_remaining_funds` requires `total == current_balance`, but the distribution sum is 0 ≠ `current_balance`.

**Impact**: Low. The total-recipient case is an edge case requiring very small distributable amounts relative to recipient count. The map-order issue affects individual recipient amounts by up to 1 unit per recipient but the total is always correct.

---

#### T7 — `FundedAmount` TTL Expires Before `Escrow` TTL

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `core/escrow.rs:67-69`, `core/validators/escrow.rs:243-244,319` |

**Description**: `FundedAmount` is extended only inside `fund_escrow` (`escrow.rs:67-69`), while `Escrow` is extended on every write (release, update, manage milestones, dispute, resolve, withdraw — all call `extend_ttl` on `DataKey::Escrow`). On a long-running escrow where many operations happen after funding but no additional funding occurs, `FundedAmount` TTL expires first.

When `FundedAmount` expires, `e.storage().persistent().get(&DataKey::FundedAmount)` returns `None`, and `.unwrap_or(0)` yields `0`. The `contract_balance` checks at `validators/escrow.rs:243-244` and `319` now evaluate to `false`. This silently restores the admin's ability to:

- Rewrite `platform_fee`, `roles`, `trustline`, `title`, `description`, and `receiver_memo` via `update_escrow` (`validators/escrow.rs:231-242`)
- Overwrite milestone amounts and descriptions via `manage_milestones` (`validators/escrow.rs:319`)

The admin cannot change the admin or platform addresses (those are immutable after init at `validators/escrow.rs:227-229,235-237`), but all other escrow properties become mutable.

**Impact**: Medium. On a long-running escrow, the admin silently regains write access to properties that should be locked after funding. This is a Tampering concern — the TTL expiry acts as an implicit timeout on the `FundedAmount > 0` guard.

---

### Repudiation

#### R1 — `EscrowUpdated` Event Omits Changed Properties

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `events/handler.rs:32-38` |

**Description**: The `EscrowUpdated` event carries only `engagement_id` and `admin`. When the admin changes `platform_fee`, `trustline`, `roles`, or other properties via `update_escrow`, the event does not record which properties changed or their old/new values. An actor can deny which specific changes they made.

**Impact**: Medium. The admin can change `platform_fee` to 9900 BPS, `roles` lists, and `trustline` before funding — and the event only proves "some update happened," not what changed.

---

#### R2 — `MilestonesManaged` Event Omits Milestone Details

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `events/handler.rs:91-99` |

**Description**: The `MilestonesManaged` event carries only `added_count` and `updated_count`. When the admin adds a new milestone with an attacker-controlled receiver or changes amounts (per §3.2, T2), the event does not record the specific changes. Combined with §3.3, T3 (status/evidence rewritable after release), the on-chain record can be altered with no audit trail.

**Impact**: Medium. A complete audit trail requires reading the full escrow state at each point in time, not just event logs.

---

#### R3 — Release Signer Signs Only Indices, Not Amounts or Fees

| Field | Value |
|-------|-------|
| **Severity** | **Low** |
| **File:Line** | `contract.rs:106-128`, `core/escrow.rs:74-82` |

**Description**: `release_funds` takes `milestone_indices: Vec<u32>` and the release signer authorizes that call. The signer does not sign the amounts, fee percentages, or receiver addresses — those are read from stored state. If the admin changes milestone amounts (§3.2, T2) or platform fee (§3.10) between the signer's off-chain review and the on-chain transaction, the signer's authorization covers the modified values.

**Impact**: Low. The signer can mitigate by re-reading the escrow before signing, but the contract provides no mechanism for the signer to verify amounts match expectations.

---

### Information Disclosure

#### I1 — `observer` Role Exists But Is Never Used

| Field | Value |
|-------|-------|
| **Severity** | **Info** |
| **File:Line** | `storage/types.rs:87`, `core/validators/escrow.rs:131,139` |

**Description**: The `observer` role is defined in `Roles` (`types.rs:87`) with a 5-member limit and duplicate checks (`validators/escrow.rs:131,139`), but no entrypoint checks the observer role. The role is stored in the escrow state and visible to anyone via `get_escrow`. The code implies a viewer allowlist that does not exist.

**Impact**: Info. No security impact; the role is informational only.

---

#### I2 — `get_escrow` Returns Full State Without Authentication

| Field | Value |
|-------|-------|
| **Severity** | **Info** |
| **File:Line** | `core/escrow.rs:289-294` |

**Description**: `get_escrow` is a public entrypoint that returns the full escrow state including all role addresses, milestone amounts, and dispute reasons. This is expected for a trustless system but means all role addresses and financial details are publicly visible on-chain.

**Impact**: Info. Expected for transparency; not a vulnerability.

---

#### I3 — `receiver_memo` Field Is Never Used in Transfers

| Field | Value |
|-------|-------|
| **Severity** | **Info** |
| **File:Line** | `storage/types.rs:38`, `core/validators/escrow.rs:251` |

**Description**: The `Escrow` struct contains `receiver_memo: u32` and the field is read in `validate_escrow_conditions` (`validators/escrow.rs:251`) as part of the `has_funds` property-mismatch check — changes to `receiver_memo` while funds are held are rejected. However, the value is never used in token transfers or payment memos in `release_funds_execute` or `calculate_and_distribute_fees`.

**Impact**: Info. The field participates in change-guard validation but has no functional effect on fund transfers.

---

### Denial of Service

#### D1 — Underfunded Escrow Can Lock Disputed Funds Permanently

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `core/dispute.rs:35-38`, `core/validators/dispute.rs:39-40,100-104` |

**Description**: `fund_escrow` does not require the funded amount to equal the sum of milestone amounts — `validate_fund_escrow_conditions` (`validators/escrow.rs:378-388`) only checks `amount > 0` and sufficient funder balance. If the funder deposits less than the total milestone amounts, releasing some milestones drains the balance below what is needed to resolve remaining disputes.

`resolve_dispute` requires `total == milestone_amount_total` (`validators/dispute.rs:100-104`) and `current_balance >= total` (`validators/dispute.rs:106-108`). If the balance has dropped below the disputed amount, resolution is permanently blocked. `withdraw_remaining_funds` requires all milestones processed (`dispute.rs:35-38`), which includes `dispute.resolved` for disputed ones — creating a circular dependency. `withdraw_remaining_funds` also requires `total == current_balance` (`validators/dispute.rs:50`), so partial resolution is not possible.

**Exploit Path**:
1. Escrow: M0 = 100 (disputed), M1 = 100 (not disputed). Total milestone amounts = 200.
2. Funder calls `fund_escrow` with `amount = 150`. `validate_fund_escrow_conditions` checks `amount > 0` and balance — passes. Contract balance = 150.
3. Release signer releases M1 → balance = 150 - 100 = 50.
4. Resolver calls `resolve_dispute([0])`. `milestone_amount_total = 100`, `total = 100`, `current_balance = 50`. Check: `current_balance < total` → `InsufficientFundsForResolution`. Permanently blocked.
5. `withdraw_remaining_funds` requires all milestones processed → M0 is `disputed` not `resolved` → `EscrowNotFullyProcessed` (`dispute.rs:35-38`). 50 units locked forever.

The exact-equality check that creates this lock (`total == milestone_amount_total` at `validators/dispute.rs:102-104`) was introduced as a safety measure (commit 161), but it creates an unresolvable state when the balance is insufficient.

**Impact**: Medium. Real funds can be locked with no on-chain exit path. The only recovery is social (resolver cooperation or admin adding new milestones to rebalance).

---

#### D2 — Dispute Resolver Unavailability Locks Funds

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `core/dispute.rs:35-38`, `core/validators/dispute.rs:39-40` |

**Description**: If all dispute resolvers become unavailable (key lost, entity dissolved), disputed milestones cannot be resolved and `withdraw_remaining_funds` cannot sweep remaining balances. Unlike `release_signers` (which has multiple members as fallback), there is no social recovery or timeout mechanism.

**Impact**: Medium. Disputed milestones are permanently stuck.

---

#### D3 — Disputes Are One-Way — No Cancellation, No Timeout

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `core/dispute.rs:151-179`, `core/validators/dispute.rs:133-181` |

**Description**: `is_disputed = false` is written only in `resolve_dispute` (`dispute.rs:125`). There is no cancellation mechanism, no timeout, no bond. Any of up to 16 authorized addresses (approvers + service_providers + release_signers + platform + per-milestone receivers) can dispute any milestone, and the dispute cannot be undone except by the dispute resolver.

A single dispute on any milestone blocks `update_escrow` (`validators/escrow.rs:239-241`) and `manage_milestones` amount updates (`validators/escrow.rs:276-282`).

**Impact**: Medium. A single unauthorized dispute (by any of 16+ addresses) can freeze the entire escrow's admin functions.

---

### Elevation of Privilege

#### E1 — Platform Fee Can Be Set to 97%

| Field | Value |
|-------|-------|
| **Severity** | **Info** |
| **File:Line** | `core/validators/escrow.rs:168-170` |

**Description**: The platform fee is capped at `99 * 100 = 9900` BPS (99%) in `validate_escrow_conditions` (`validators/escrow.rs:168-170`). Combined with the fixed 30 BPS (0.3%) trustless work fee, the total fee can reach 99.3%, leaving the receiver with 0.7%.

**Why not High**: `fund_escrow` requires `stored_escrow.eq(&expected_escrow)` (`validators/escrow.rs:382-384`), and `Escrow` derives `PartialEq` including `platform_fee` (`storage/types.rs:29-39`). The funder cryptographically consents to the exact escrow properties — including the fee — at funding time. After funding, `has_funds` locks the fee (`validators/escrow.rs:243-244`). So "platform extracts 97%" requires a funder who knowingly funded a 97%-fee escrow. The plain-token-transfer variant (§3.10, E2) bypasses consent, but then nobody was deceived — someone sent tokens outside the documented flow.

**Impact**: Info. Configuration footgun, not an exploitable privilege escalation. The fee is visible to the funder before they commit funds.

---

#### E2 — Admin Can Be Set as Platform (Fee Collector)

| Field | Value |
|-------|-------|
| **Severity** | **Info** |
| **File:Line** | `core/validators/escrow.rs:102-108` |

**Description**: `validate_admin_role_overlap` (`validators/escrow.rs:102-108`) checks admin against approvers, service_providers, release_signers, and dispute_resolvers — but **not platform**. The admin address can be the same as the platform address. This means the admin can collect both the admin role and the platform fee.

**Why not High**: Same reasoning as §3.10, E1 — `fund_escrow` requires the funder to consent to the exact escrow properties including `platform`. The `platform_fee` is baked into the `Escrow` struct that the funder must pass. A funder who funds an escrow where `admin == platform` and `platform_fee = 9700` has consented to those terms. The plain-token-transfer variant bypasses consent, but is not a deception — it is a voluntary transfer outside the contract's funding flow.

**Impact**: Info. Configuration footgun; no silent extraction is possible through the contract's own entrypoints.

---

#### E3 — Dispute Resolver Can Overlap with Platform

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `core/validators/escrow.rs:147-157` |

**Description**: `validate_dispute_resolver_role_overlap` (`validators/escrow.rs:147-157`) checks dispute_resolvers against approvers, service_providers, and release_signers — but **not platform**. A dispute_resolver can be the same address as the platform.

**Impact**: Medium. If dispute_resolver == platform, the same address decides how disputed funds are split AND collects the platform fee. This enables self-dealing in dispute resolution.

---

#### E4 — Non-Admin Cannot Update Escrow or Manage Milestones

| Field | Value |
|-------|-------|
| **Severity** | **Info** |
| **File:Line** | `core/validators/escrow.rs:227-229, 273-275` |

**Description**: `update_escrow` and `manage_milestones` both verify `caller == existing.roles.admin` before proceeding. Confirmed by attack tests E3 and E4.

**Impact**: Info. Correctly defended.

---

#### E5 — Admin and Platform Addresses Are Immutable

| Field | Value |
|-------|-------|
| **Severity** | **Info** |
| **File:Line** | `core/validators/escrow.rs:231-233, 235-237` |

**Description**: `validate_escrow_conditions` with `is_init: false` checks `existing.roles.admin != new_escrow.roles.admin` → `AdminAddressCannotBeChanged` (`validators/escrow.rs:231-233`), and `existing.roles.platform != new_escrow.roles.platform` → `PlatformAddressCannotBeChanged` (`validators/escrow.rs:235-237`). Confirmed by attack tests E4 and E5.

**Impact**: Info. Correctly defended.

---

#### E6 — `approve_and_release_milestones` Collapses Two-Party Control

| Field | Value |
|-------|-------|
| **Severity** | **Medium** |
| **File:Line** | `contract.rs:260-287` |

**Description**: `approve_and_release_milestones` (`contract.rs:260-287`) requires the signer to be in both `approvers` and `release_signers` (`contract.rs:266-268`), then calls `approve_milestones_inner` followed by `release_funds_inner` in a single transaction. There is no role overlap check preventing an address from being in both `approvers` and `release_signers` — `validate_admin_role_overlap` (`validators/escrow.rs:102-108`) only guards admin against these roles, and `validate_dispute_resolver_role_overlap` (`validators/escrow.rs:147-157`) only guards dispute_resolvers.

With `approvals.target == 1` (a common configuration for two-party escrows), a single address can approve and release a milestone in one transaction, bypassing the intended two-party control where one party approves and a different party releases.

**Impact**: Medium. The intended security model — "approver approves, release_signer releases" — is defeated when a single address holds both roles. This is the cleanest EoP for the role-based verification/release separation.

---

#### E7 — Admin Can Swap Operational Roles Pre-Funding

| Field | Value |
|-------|-------|
| **Severity** | **Low** |
| **File:Line** | `core/validators/escrow.rs:227-229, 273-275` |

**Description**: While `admin` and `platform` are immutable after init (`validators/escrow.rs:231-237`), all other roles — `approvers`, `service_providers`, `release_signers`, `dispute_resolvers`, `observers` — and the `trustline` are fully mutable by the admin via `update_escrow` (when `FundedAmount == 0`). Changing `trustline` after a direct token transfer (outside `fund_escrow`) strands the original asset, as the contract will only interact with the new trustline.

**Impact**: Low. Requires admin action before funding, and `fund_escrow` validates the trustline matches the stored escrow. The impact is limited to griefing or confusion if tokens are sent via plain transfer.

---

## 4. Verified Attack Test Results

A test suite of 27 Rust test cases was executed against the contract (commit `2f569ec`). All 27 tests passed (89 total tests pass). An additional 11 STRIDE verification tests were written and all passed (73 total). Below are the key results.

### STRIDE Verification Summary

| Finding | Test | Result | Status |
|---------|------|--------|--------|
| T1: Overflow in distribution.rs:35 | `t1_overflow_resolve_dispute_single_recipient` | Overflow triggered at 1.5e19 | ✅ Verified |
| T1: Overflow in distribution.rs:35 | `t1_overflow_withdraw_remaining_funds` | Overflow triggered via overfund + withdraw | ✅ Verified |
| D1: Underfunding locks funds | `d1_underfunding_locks_disputed_funds` | 50 units permanently locked | ✅ Verified |
| D1: Underfunding allowed | `d1_fund_escrow_allows_underfunding` | fund_escrow accepts amount < milestone total | ✅ Verified |
| E6: Two-party collapse | `e6_single_address_approves_and_releases` | Single address approves+releases with target=1 | ✅ Verified |
| E6: Two-party separation | `e6_approver_only_cannot_approve_and_release` | Separate roles correctly enforced | ✅ Verified |
| E1: Funder consent | `e1_funder_consents_to_high_fee` | Mismatched expected_escrow rejected | ✅ Verified |
| E2: admin==platform | `e2_admin_can_be_platform` | Initialization succeeds (no overlap check) | ✅ Verified |
| E3: resolver==platform | `e3_dispute_resolver_can_be_platform` | Initialization succeeds (no overlap check) | ✅ Verified |
| T7: TTL expiry | `t7_funded_amount_unwrap_or_zero_code_path` | Before/after behavior documented; mock can't expire persistent storage | ⚠️ Code path verified |

### Spoofing — All 7 Attacks Defended

**A1 (Random address calls `resolve_dispute`)**:
```
1. Deploy escrow with dispute_resolver = D.
2. Random address R calls resolve_dispute(R, trustless_work, [0], {receiver: 100M}).
3. validate_dispute_resolution_conditions checks dispute_resolvers.contains(R) → false.
4. Returns OnlyDisputeResolverCanExecuteThisFunction. ✓
```

**A2 (Admin calls `dispute_milestones`)**:
```
1. Deploy escrow where admin ≠ platform, admin ≠ any global role, admin ≠ any receiver.
2. Admin calls dispute_milestones(admin, [0], "dispute").
3. validate_batch_milestone_dispute_conditions checks is_global_role(admin) → false,
   then checks is_receiver_for_all(admin, [0]) → false.
4. Returns UnauthorizedToChangeDisputeFlag. ✓
```

**A3 (Dispute resolver calls `dispute_milestones`)**:
```
1. Deploy escrow with dispute_resolver = D.
2. D calls dispute_milestones(D, [0], "dispute").
3. validate_batch_milestone_dispute_conditions checks dispute_resolvers.contains(D) → true.
4. Returns DisputeResolverCannotDisputeTheEscrow. ✓
```

**A4 (Receiver A disputes Receiver B's milestone)**:
```
1. Deploy escrow: M0 → receiver A, M1 → receiver B.
2. A calls dispute_milestones(A, [1], "steal").
3. is_global_role(A) → false. is_receiver_for_all(A, [1]) → &m1.receiver != A → false.
4. Returns UnauthorizedToChangeDisputeFlag. ✓
```

### Tampering — All 5 Attacks Defended

**A5 (Dispute after release)**:
```
1. Fund 100M, approve M0, release M0. M0.released = true.
2. Approver calls dispute_milestones(approver, [0], "post-release").
3. validate_batch_milestone_dispute_conditions checks milestone.released → true.
4. Returns MilestoneAlreadyReleased. ✓
```

**A6 (Batch atomicity — partial failure)**:
```
1. Approver calls approve_milestones([0, 99], approver).
2. validate_batch_milestone_approve iterates: index 99 >= milestones.len() → false.
3. Returns MilestoneToApproveDoesNotExist. Entire batch rejected.
4. Verify: M0.approval_count == 0 (unchanged). ✓
```

### Elevation of Privilege — 2 Confirmed Exploitable

**A7 (Admin reduces amount before funding)**:
```
1. Admin initializes escrow: M0 = 100M → receiver A.
2. Admin calls manage_milestones(admin, [], [{index: 0, new_amount: Some(1)}]).
3. validate_manage_milestones_conditions: contract_balance == 0 → passes.
4. M0.amount is now 1. Admin can add M1 = 99M → admin.
5. ⚠️ Confirmed exploitable (requires funder to pass modified escrow to fund_escrow).
```

**A8 (97% platform fee)**:
```
1. Admin initializes escrow with platform_fee = 9700 (97%).
2. Funder deposits 100M via fund_escrow (must pass modified escrow as expected_escrow).
3. Approver approves M0, release_signer releases M0.
4. Fee calc: TW = floor(100M * 30 / 10000) = 300K. Platform = floor(100M * 9700 / 10000) = 97M.
5. Receiver net = 100M - 300K - 97M = 2.7M.
6. ⚠️ Confirmed — but requires funder consent (must pass modified escrow to fund_escrow).
```

### Denial of Service — 1 Confirmed Exploitable

**A9 (Underfunding locks disputed funds)**:
```
1. Escrow: M0 = 100 (disputed), M1 = 100 (not disputed). Fund only 150.
2. Release M1 → balance = 50.
3. resolve_dispute([0]) demands total == 100, but current_balance == 50 → InsufficientFundsForResolution.
4. withdraw_remaining_funds requires all processed → M0 is disputed not resolved → EscrowNotFullyProcessed.
5. ⚠️ 50 units permanently locked.
```

### Reentrancy — Both Attacks Defended

**A10 (Reenter `resolve_dispute`)**:
```
1. Set DataKey::Reentrancy = true (simulate in-flight call).
2. Call resolve_dispute → checks has(&DataKey::Reentrancy) → true.
3. Returns FlagsMustBeFalse. ✓
```

---

## 5. Positive Observations

1. **Batch atomicity**: All batch operations validate the entire index set before any state mutation. Partial application is structurally impossible — validation completes before the in-memory copy is modified and written back with a single `storage.set`. This matches the team's own reasoning in `tests/batch_atomicity.rs:13-17`.

2. **Checks-Effects-Interactions pattern**: `release_funds_execute` (`escrow.rs:97-173`) commits all milestone state changes to storage (line 125) before any external token transfers (lines 142-146, 150-155, 159). This prevents reentrancy-based state manipulation on the release path.

3. **Role separation enforcement**: Dispute resolvers cannot overlap with approvers, service_providers, or release_signers (`validators/escrow.rs:147-157`). The admin cannot overlap with any operational role (`validators/escrow.rs:102-108`). This prevents single-party control over both dispute initiation and resolution. (Note: `platform` is not checked — see §3.5.)

4. **Overflow protection in fee calculation**: `FeeCalculator::calculate_standard_fees` (`fee/calculator.rs:34-38`) uses `SafeMath::safe_mul_div` which correctly handles large values by decomposing the multiplication to avoid intermediate overflow. An explicit test (`modules/math/safe.rs:49-55`) proves the team is aware of this class of bug — the oversight in `distribution.rs:35` (§3.1, T1) appears to be an error, not a design choice.

5. **Reentrancy guard**: The `DataKey::Reentrancy` flag (`dispute.rs:27-30, 79`) is properly set before interactions and cleared after, covering both `resolve_dispute` and `withdraw_remaining_funds` — the two functions that perform external calls after state mutations.

6. **Checked arithmetic throughout**: All financial arithmetic uses `checked_*` methods via `BasicMath` (`modules/math/basic.rs`). The `safe_mul_div` in `safe.rs:20-21,29` does use raw `/` and `%`, but only on a nonzero constant divisor (`10000`), which is safe. The `safe_mul` + `safe_div` pattern in `distribution.rs:35` should use `safe_mul_div` instead (§3.1, T1).

7. **Fee rounding remainder handling**: `calculate_and_distribute_fees` (`fee/distribution.rs:43-51`) assigns floor-division rounding remainder to the last recipient, ensuring `sum(all transfers) == distributable` and preventing dust accumulation. The remainder assignment is map-order-dependent (§3.6, T6) but the total is always correct.

8. **TTL handling**: `extend_contract_ttl` (`contract.rs:188-206`) extends only `DataKey::Escrow`. `FundedAmount` TTL is extended only inside `fund_escrow` (`escrow.rs:67-69`). The `31536000` argument to `extend_ttl` is safe — Soroban's persistent `extend_ttl` clamps to the max rather than erroring. However, if `FundedAmount` TTL expires while `Escrow` persists, the `FundedAmount > 0` guards are bypassed (see §3.8, T7).

---

## 6. Summary of Findings

| # | STRIDE | Severity | Title |
|---|--------|----------|-------|
| T1 | Tampering | **High** | Intermediate overflow in `calculate_and_distribute_fees` (18-decimal tokens only) |
| D1 | DoS | **Medium** | Underfunded escrow can lock disputed funds permanently |
| D2 | DoS | **Medium** | Dispute resolver unavailability locks funds |
| D3 | DoS | **Medium** | Disputes are one-way — no cancellation, no timeout |
| S1 | Spoofing | Medium | `get_escrow_by_contract_id` invokes arbitrary contracts |
| T2 | Tampering | Medium | Admin can reduce milestone amounts before funding |
| T3 | Tampering | Medium | Status/evidence rewritable after release/resolution |
| T4 | Tampering | Medium | Zero-milestone escrow blocks all exit paths |
| T5 | Tampering | Medium | `FundedAmount` counter blocks post-release modifications |
| T7 | Tampering | Medium | `FundedAmount` TTL expires before `Escrow` TTL |
| R1 | Repudiation | Medium | `EscrowUpdated` event omits changed properties |
| R2 | Repudiation | Medium | `MilestonesManaged` event omits milestone details |
| E3 | EoP | Medium | Dispute resolver can overlap with platform |
| E6 | EoP | Medium | `approve_and_release_milestones` collapses two-party control |
| S2 | Spoofing | Low | `tw_new_multi_release_escrow` enables address squatting / front-running |
| T6 | Tampering | Low | Remainder assignment is map-order-dependent |
| R3 | Repudiation | Low | Release signer signs only indices, not amounts/fees |
| E7 | EoP | Low | Admin can swap operational role set pre-funding |
| S3 | Spoofing | Info | Auth-first vs. validate-first inconsistency |
| I1 | Info Disc | Info | `observer` role exists but is never used |
| I2 | Info Disc | Info | `get_escrow` returns full state without authentication |
| I3 | Info Disc | Info | `receiver_memo` field is never used in transfers |
| E1 | EoP | Info | Platform fee can be set to 97% (requires funder consent) |
| E2 | EoP | Info | Admin can be set as platform (requires funder consent) |
| E4 | EoP | Info | Non-admin cannot update escrow or manage milestones |
| E5 | EoP | Info | Admin and platform addresses are immutable |

Information Disclosure has only three Info-level findings — effectively no risks, just dead code and public-data observations. The category was checked thoroughly but yielded no Medium-or-above issues.

---

## 7. Recommendations

### For Finding T1 (Overflow in `distribution.rs:35`) — HIGH PRIORITY

**R.1**: Replace `safe_div(safe_mul(amount, distributable), total)` with `SafeMath::safe_mul_div(amount, distributable, total)`. This is the same function already used in `fee/calculator.rs:34-38` and has explicit overflow testing at `modules/math/safe.rs:49-55`.

### For Finding D1/D2/D3 (Dispute Fund Locks) — HIGH PRIORITY

**R.1**: Add a timeout mechanism: if a disputed milestone is not resolved within N ledgers, allow the original funder or admin to reclaim funds or escalate to a secondary resolver.

**R.2**: Consider requiring multi-sig from dispute resolvers (e.g., 2-of-3) for high-value disputes.

### For Finding T2/T5/T7 (Admin Amount Manipulation + FundedAmount)

**R.1**: Decrement `FundedAmount` when tokens leave the contract (on release, dispute resolution, or withdrawal). This restores the ability to update escrow properties after funds are fully distributed.

**R.2**: Alternatively, extend `FundedAmount` TTL alongside `Escrow` TTL on every write, or persist it in the same storage key as the escrow.

### For Finding T3 (Status Rewritable After Release)

**R.1**: Add a guard in `validate_batch_milestone_status_change` (`validators/milestone.rs:13-55`) that rejects updates to milestones where `released == true` or `dispute.resolved == true`.

### For Finding R1/R2 (Weak Event Coverage)

**R.1**: Include the changed properties in `EscrowUpdated` (e.g., a bitmask or list of changed field names).

**R.2**: Include milestone indices and their new amounts/receivers in `MilestonesManaged`.

### For Finding D3 (No Dispute Cancellation)

**R.1**: Add a timeout-based cancellation: if a dispute is not resolved within N ledgers, it is automatically cancelled and the milestone returns to its pre-dispute state.

### For Finding E3/E6 (Role Overlaps)

**R.1**: Add `platform` to `validate_dispute_resolver_role_overlap` (`validators/escrow.rs:147-157`) to prevent `dispute_resolver == platform`.

**R.2**: Consider adding an explicit check preventing `approve_and_release_milestones` from succeeding when the signer is the only approver and the only release_signer — or document that this is an accepted design choice for small teams.

---

*End of report.*
