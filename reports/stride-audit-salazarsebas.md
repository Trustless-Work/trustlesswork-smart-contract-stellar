# STRIDE threat model: single-release escrow

- **Author:** salazarsebas
- **Branch reviewed:** `single-release-develop-v2` (`beb9a4d`)
- **Scope:** `contracts/escrow/src/**` only
- **Method:** Stellar STRIDE template (https://developers.stellar.org/docs/build/security-docs/threat-modeling/STRIDE-template) applied to each public entrypoint and to the storage/TTL/fee paths the issue called out
- **Tools:** manual review of source + validators + fee/math modules. No scanner-only output.

This is an independent reading of the **single-release** model (one escrow-level `amount` and `receiver`, milestones as release gates, one escrow-level dispute). It is not a comparison with the multi-release variant.

---

## 1. Scope and methodology

**In scope**

| Area | Paths |
| --- | --- |
| Entrypoints | `contracts/escrow/src/contract.rs` |
| Core logic | `core/escrow.rs`, `core/milestone.rs`, `core/dispute.rs` |
| Validators | `core/validators/{escrow,milestone,dispute}.rs` |
| Fees / math | `modules/fee/{calculator,distribution}.rs`, `modules/math/{basic,safe}.rs` |
| Storage / events | `storage/types.rs`, `events/handler.rs`, `error.rs` |

**Out of scope (per issue):** off-chain API, deployment tooling, other branches.

For each STRIDE category I walked:

1. Every public function in `EscrowContract`
2. The stored `Roles` / `Dispute` / `FundedAmount` / TTL state it reads or writes
3. Whether `require_auth()` is bound to an address loaded from storage (or checked against a stored role list) rather than an arbitrary parameter
4. A realistic on-chain exploitation path, or an explicit "nothing found" with what was checked

Severity uses impact × likelihood in this contract's actual trust model (not generic DeFi slogans):

| Severity | Meaning |
| --- | --- |
| High | Direct loss of escrow principal or unauth role capture under the documented model |
| Medium | Loss of protocol/platform fees leftover funds or availability of a funded escrow without needing the full trusted-role set |
| Low | Integrity/availability issue that needs an already-trusted actor extra operational failure or does not move principal |
| Info | Trust assumption documentation gap or defense-in-depth note |

---

## 2. Trust model (as the code communicates it)

### Roles (`storage/types.rs` `Roles`)

| Role | On-chain power |
| --- | --- |
| `admin` | `initialize` handoff target after constructor; `update_escrow`; `manage_milestones`; `extend_contract_ttl`. Cannot overlap approver / SP / release signer / dispute resolver / receiver. Cannot be changed after init. |
| `approvers` | Approve milestones (threshold `MilestoneApprovals.target`). Can open a dispute. |
| `service_providers` | Set milestone `status` / `evidence` strings. Can open a dispute. |
| `release_signers` | Call `release_funds` once every milestone meets `approval_count >= target`. Can open a dispute. |
| `dispute_resolvers` | `resolve_dispute` (arbitrary distribution of **current token balance**) and `withdraw_remaining_funds` after release or resolution. Cannot open a dispute. Cannot overlap the other operational roles or the receiver. |
| `receiver` | Single payout address on happy-path release. Can open a dispute. Not admin. |
| `platform` | Receives `platform_fee` bps. Can open a dispute. Address frozen after init. |
| `observers` | Stored only. No entrypoint checks this list. |

Constructor stores a bootstrap `DataKey::Admin` plus `DataKey::ApprovedWasmHash`. `initialize_escrow` requires that bootstrap admin's auth then **deletes both keys**. After init this instance is no longer a factory.

Milestones do **not** carry amounts or receivers. They are gates: `validate_release_conditions` requires every milestone `approval_count >= target` (`core/validators/escrow.rs`). Status/evidence strings are not part of the release predicate.

Happy-path money movement is `escrow.amount` (not `FundedAmount` and not full token balance). Dispute/withdraw move **full current token balance** via caller-supplied `distributions`.

Trusted off-chain parties the code cannot see: whoever holds the dispute-resolver keys; whoever is configured as `trustless_work_address` at release time; the token contract at `trustline.address`.

---

## 3. Findings

### Spoofing

#### S.1 Factory entrypoint is not bound to stored admin — Medium

- **Where:** `contracts/escrow/src/contract.rs:32-64`
- **What:** `tw_new_single_release_escrow` allowlists WASM (`wasm_hash == DataKey::ApprovedWasmHash`) and calls `signer.require_auth()`. It never compares `signer` to `DataKey::Admin`. `constructor_args` and `init_fn` / `init_args` are fully caller-chosen so the caller picks the child admin and the function invoked after `deploy_v2`.
- **Why it matters:** Error `OnlyAdminAddressExecuteThisFunction` is used for a WASM mismatch (`contract.rs:49-53`) which implies an admin-gated factory. The implementation is a **permissionless clone factory** of the approved WASM for as long as the parent has no `DataKey::Escrow`. Anyone can spawn children and become their constructor admin.
- **Exploit path:** Keep one parent uninitialized (the documented factory). Call `tw_new_single_release_escrow` with your own address in `constructor_args` and `initialize_escrow` in `init_fn`. You now operate a legitimate-looking child escrow. This is not WASM injection (hash is checked) but it is impersonation of "official" factory output if indexers/UI assume only the bootstrap admin can deploy.
- **If this is the intended model:** say so in docs and stop returning `OnlyAdminAddressExecuteThisFunction` for non-admin failures. If it is not intended: `stored_admin.require_auth()` before `deploy_v2`.

#### S.2 Role auth on fund/release/dispute/approve — nothing additional found

Checked: `fund_escrow` auths the token sender; `release_funds` checks `release_signers.contains` then `require_auth`; `approve_milestones` checks `approvers.contains` then `require_auth`; `dispute_escrow` allowlists operational roles and rejects dispute resolvers; `resolve_dispute` / `withdraw_remaining_funds` check `dispute_resolvers.contains` then `require_auth`; `initialize_escrow` auths stored constructor admin; `update_escrow` / `manage_milestones` / `extend_contract_ttl` auth `roles.admin`. Duplicate addresses inside a role list are rejected. Admin/DR overlap with operational roles is rejected.

No "auth an arbitrary `Address` parameter and treat that as authorization" pattern on the money paths except the factory finding above and `trustless_work_address` (see E.1).

---

### Tampering

#### T.1 `FundedAmount` not the token balance so property locks can be skipped — Medium

- **Where:** `core/escrow.rs:144-155` and `180-191` pass `DataKey::FundedAmount` into validators as `contract_balance`. `core/validators/escrow.rs:233-246` only freezes engagement/roles/amount/fee/trustline when that value is `> 0`. `validate_manage_milestones_conditions` (`:295-297`) blocks description updates the same way.
- **What:** `fund_escrow` is the only path that increments `FundedAmount` (`core/escrow.rs:56-64`). A SAC `transfer` straight into the contract credits tokens without touching that key.
- **Exploit path:**
  1. Init escrow `amount = 1000`.
  2. Payer (or anyone) transfers 1000 of the trustline token to the contract without calling `fund_escrow`. `FundedAmount` stays 0.
  3. Admin calls `update_escrow` and changes `amount` / `receiver` / `release_signers` because the "has funds" lock is false.
  4. After colluding approvals `release_funds` pays `escrow.amount` from the real token balance (`core/escrow.rs:107-133`).
- **Impact:** Principal can be redirected if the payer does not use `fund_escrow`. The TOCTOU `expected_escrow` check on `fund_escrow` does not help this path.
- **Mitigation:** Treat `token.balance(contract)` (and/or `FundedAmount`) as the lock. Reject `update_escrow` / milestone description edits when either is non-zero. Optionally reject `fund_escrow` unless it is the only accepted deposit path and document that raw transfers are unaccounted.

#### T.2 Milestone `status` / `evidence` are unconstrained and ignored at release — Low

- **Where:** `core/validators/escrow.rs:41-44` and `:67-68`; `core/validators/milestone.rs:12-54`; `core/milestone.rs:13-49`.
- **What:** Release only tests `approval_count >= target`. A service provider can set `status` to any string ≤ 50 chars including during a dispute (no dispute check on `change_milestone_status`). Evidence is optional free text.
- **Exploit path:** UI that keys off `status == "completed"` can be lied to while approvals (the real gate) are unchanged. Not a principal theft by itself.
- **Mitigation:** If status is meant to be a gate enumerate allowed values and include them in `validate_release_conditions`. If it is decorative document that clearly to frontend authors.

#### T.3 Effects-before-interactions on release/resolve — nothing found

`release_funds_execute` sets `released = true` before transfers (`core/escrow.rs:98-102`). `resolve_dispute` sets `resolved` / clears `is_disputed` before transfers (`core/dispute.rs:117-123`). Failed transfers revert the invocation so those flags are not stuck. Host-level reentrancy is blocked; `DataKey::Reentrancy` on resolve/withdraw is extra.

---

### Repudiation

#### R.1 Evidence updates are not in the milestone-status event — Low

- **Where:** `events/handler.rs:47-54` (`MilestoneStatusChanged` carries `Vec<MilestoneStatusEntry>`). `storage/types.rs:4-8` (`MilestoneStatusEntry` is `{ index, status }` only). `contract.rs` `change_milestone_status` copies status not evidence into the event.
- **What:** Evidence is persisted on the `Escrow` blob so it is reconstructible from storage snapshots. Off-chain indexers that only consume events cannot prove who attached which evidence when.
- **Mitigation:** Add evidence (or its hash) to the event payload.

#### R.2 Happy-path and dispute money events — nothing additional found

Present and attributed: `InitEsc`, `FundEsc` (includes `funder` and `funded_total`), `ReleaseEsc` (signer receiver amount fees), `EscrowUpdated`, `MilestonesManaged`, `MilestonesApproved`, `EscrowDisputed` (signer + reason), `DisputeResolved` (resolver + net distributions), `FundsWithdrawn`, `TtlExtended`. Fund accounting is observable from `FundEsc.funded_total` plus storage of `FundedAmount`.

---

### Information disclosure

#### I.1 Dispute reason and full escrow blob are public — Info

Expected on Soroban. `dispute.reason` (up to 500 chars, `core/dispute.rs:144-147`) and all roles live in persistent `DataKey::Escrow`. Anyone can `get_escrow`. Not a vulnerability for this product; parties should assume the reason is public.

#### I.2 `get_escrow_by_contract_id` / `get_multiple_escrow_balances` invoke arbitrary addresses — Low

- **Where:** `core/escrow.rs:216-249`
- **What:** Up to 20 addresses. For each non-self address the contract does `invoke_contract::<Escrow>(id, "get_escrow", [])` then `token.balance`. No allowlist.
- **Impact:** The caller pays the budget. A crafted destination can fail or be expensive. It cannot drain this escrow. Cap of 20 limits blast radius.
- **Mitigation:** Only accept addresses that share the approved WASM or drop the cross-contract helper if the API can query children directly.

---

### Denial of service

#### D.1 `extend_contract_ttl` does not cover `FundedAmount` or instance storage — Medium

- **Where:** `contract.rs:176-191` extends only `DataKey::Escrow`. `FundedAmount` TTL is extended solely inside `fund_escrow` (`core/escrow.rs:65-67`). No `env.storage().instance().extend_ttl` anywhere.
- **What:** Persistent `FundedAmount` can archive while `Escrow` is kept alive by admin TTL extension or by later Escrow writes. Archived persistent entries need restore before `get`. Instance archival makes the whole contract unusable until restored. Anyone can *extend* TTLs with `ExtendFootprintTTLOp` (TTL is not a security boundary) but nobody is *required* to extend `FundedAmount` on the admin path.
- **Exploit / failure path:** Fund once. Stop calling `fund_escrow`. Admin keeps extending `Escrow`. After `FundedAmount` archives `update_escrow` / `manage_milestones` reads it (`unwrap_or(0)` does not help if the get traps on archived) and those admin paths fail. Release still uses token balance so payout may still work; property updates and further funding accounting do not.
- **Mitigation:** `extend_contract_ttl` should extend `FundedAmount` when present and extend instance TTL. Restore path documented for operators.

#### D.2 Leftover tokens after release are DR-gated — Low

- **Where:** `release_funds_execute` pays `escrow.amount` not `token.balance` (`core/escrow.rs:107-133`). `withdraw_remaining_funds` requires a dispute resolver (`core/validators/dispute.rs:20-22`) and `released || dispute.resolved` (`core/dispute.rs:36`).
- **What:** Overfunding via repeated `fund_escrow` (no cap against `escrow.amount`) or extra direct transfers leaves remainder only the DR can sweep. If every DR key is lost remainder is stuck.
- **Mitigation:** Cap `FundedAmount + amount <= escrow.amount` on `fund_escrow` or allow receiver/admin sweep of dust after `released`.

#### D.3 Bounded loops — nothing additional found

Role lists capped at 5; milestones at 50; distributions at 50; batch milestone ops at 50; `get_multiple_escrow_balances` at 20; string lengths capped. Duplicate-address checks are O(n²) with n ≤ 5. Dispute reason capped at 500.

---

### Elevation of privilege

#### E.1 Caller chooses `trustless_work_address` on every payout — Medium

- **Where:** `contract.rs` `release_funds`, `approve_and_release_milestones`, `resolve_dispute`, `withdraw_remaining_funds`. Fee send: `core/escrow.rs:114-120` and `modules/fee/distribution.rs:60-66`.
- **What:** The 30 bps Trustless Work fee (`modules/fee/calculator.rs:9-38`) is transferred to an argument not to a stored allowlisted address. A release signer or dispute resolver can point it at themselves.
- **Exploit path:** Reach `release_funds` as a legitimate release signer (approvals already done). Pass your own address as `trustless_work_address`. You receive 0.30% of `escrow.amount` in addition to whatever the receiver path pays. Same for `resolve_dispute` on the full balance.
- **Impact:** Not the principal (receiver still gets `amount - platform - 30bps` on happy path) but it is protocol-fee theft by an already-authorized signer. Off-chain API may overwrite the argument; that is out of scope. On-chain the parameter is trusted blindly.
- **Mitigation:** Store `trustless_work_address` at init (or bind it to constructor) and ignore the caller argument or require it to match storage.

#### E.2 Dispute resolver may assign the entire balance including to self — Info (trust assumption)

- **Where:** `core/validators/dispute.rs:48-79` only checks DR membership, disputed flag, and `sum(distributions) == token.balance`. No restriction on keys in the map.
- **What:** This is the explicit dispute model. A malicious or compromised DR can send 100% to themselves (minus computed fees which they can also redirect per E.1).
- **Mitigation:** Product-level: DR key hygiene maybe a delay / second signer. Code-level optional: forbid DR address in `distributions` or require receiver to appear.

#### E.3 Approve-and-release dual-role check — nothing found (positive)

`approve_and_release_milestones` (`contract.rs`) requires the signer to be in **both** `approvers` and `release_signers` before approving and releasing. Combined with admin/DR overlap bans this is the right shape.

---

## 4. Category roll-up

| Category | Result |
| --- | --- |
| Spoofing | S.1 factory not admin-bound. Other role/auth paths checked; no extra finding. |
| Tampering | T.1 FundedAmount vs token balance. T.2 status/evidence not part of release. Effects-before-interactions OK. |
| Repudiation | R.1 evidence missing from events. Money-movement events otherwise adequate. |
| Information disclosure | I.1 public dispute reason (expected). I.2 arbitrary cross-contract reads on the balance helper. |
| Denial of service | D.1 FundedAmount / instance TTL gap. D.2 leftover funds DR-gated. Loops bounded. |
| Elevation of privilege | E.1 caller-chosen protocol fee destination. E.2 DR discretion is the trust model. Dual-role release check is sound. |

No High finding against escrow **principal** under the documented happy path (payer uses `fund_escrow` approvals reach target release signer is honest). Medium findings are factory policy fee destination accounting lock and TTL.

---

## 5. Recommendations (conceptual; no code in this PR)

1. Gate `tw_new_single_release_escrow` on `DataKey::Admin` **or** document it as a permissionless WASM-allowlisted factory and fix the error names.
2. Persist `trustless_work_address` at init; payouts must use the stored value.
3. Lock admin mutations on `token.balance(contract) > 0` not only `FundedAmount`. Prefer rejecting unexpected inbound transfers or folding them into `FundedAmount`.
4. Extend `FundedAmount` and instance TTL from `extend_contract_ttl` (and from release/dispute writes).
5. Put evidence (or a hash) on `MilestoneStatusChanged`. Decide whether milestone `status` is a release gate; if yes enumerate it.
6. Cap funding at `escrow.amount` (or document overfund + DR sweep as the recovery path).
7. Keep the DR trust assumption explicit in operator docs: a live DR key can redirect the whole balance.

---

## 6. Optional proof sketches (not committed as tests)

**E.1:** Init + fund + approve all milestones. Call `release_funds(release_signer, attacker)`. Assert attacker token balance increased by `safe_mul_div(amount, 30, 10000)` and `roles.platform` / `roles.receiver` still received their slices.

**T.1:** Init `amount=1000`. `token.transfer(payer, contract, 1000)` without `fund_escrow`. As admin `update_escrow` with `amount=1` and `receiver=admin_colluder`. Should succeed today; after a fix it should return `EscrowPropertiesMismatch` or equivalent.

**S.1:** On an uninitialized parent call `tw_new_single_release_escrow` as a non-admin signer with approved WASM. Child deploy succeeds today.

---

## 7. Review notes

- Arithmetic on fees uses `SafeMath::safe_mul_div` (no intermediate `amount * bps` overflow) and remainder dust is assigned to the last dispute recipient so `sum(transfers) == total`. That path looks correct after `47ae02c`.
- `platform_fee` is capped so platform + 30 bps cannot exceed 100% (`core/validators/escrow.rs:148-155`).
- I did not run `cargo scout-audit` / Certora; findings above are from reading this branch.
