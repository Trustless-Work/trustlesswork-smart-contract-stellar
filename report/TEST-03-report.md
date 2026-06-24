## TEST-03 — Batch Operations: Atomicity and Partial Failure Behavior — Results

Validated against the **Multi-Release V2** contract (`feat/multi-release-v2`) with a
reproducible Rust integration suite — `contracts/escrow/src/tests/batch_atomicity.rs` —
run in the Soroban test environment (`soroban-sdk` 26.0.0). Each scenario deploys a fresh
escrow, drives it to the failure condition described in the issue, asserts the **exact
`#[contracterror]`** the contract returns, and then reads the escrow back
(`get_escrow`) plus the relevant token balances to prove that *nothing* was partially
applied. Reproduce with:

```
cargo test -p escrow batch_atomicity::
```

**Why this method for an atomicity test.** The single most important question here —
"did the batch apply partial changes before failing?" — is answered most reliably by
inspecting post-failure contract state deterministically, which is exactly what these
tests do. The amounts use the token's smallest unit (USDC, 7 decimals, so
`10_000_000` units = `1.0000000` USDC) and are multiples of `10_000`, so every fee split
is exact with no rounding ambiguity.

**Scope / caveat.** This is a **contract-level** verification of atomicity, not a live
HTTP-API run, so there are no on-chain transaction hashes below. It proves the on-chain
logic is atomic. It does **not** exercise the API serialization layer — in particular how
the API renders these error codes, and how it handles a literal `milestone_index: []`.
TEST-01 already found the API can surface a *misleading* message for an unrelated case, so
the API-layer wording for these errors should still be confirmed on Testnet with the
provided credentials. See "API-layer follow-ups" at the end.

**Summary:** A Pass · B Pass · C Pass · D Pass · E Pass · F Pass.
**Every scenario was atomic. No case applied partial changes before failing.**

The mechanism is the same across all three batch entry points: each one runs a
`validate_batch_*` pass over the **entire** index set first, then mutates an in-memory copy
of the escrow, then commits it with a **single** `storage.set`. Any failure returns before
that single write, so partial application is structurally impossible. Soroban's
transaction-level rollback (a returned `Err` reverts all storage in the call) is a second
line of defense.

---

```
Scenario A — Batch approve with an out-of-range index:
- Action executed: approve_milestones(milestone_index=[0, 1, 99], approver)
  on a funded escrow of 3 milestones (M0, M1, M2), all status "Completed".
- Expected result: whole batch rejected; M0 and M1 NOT approved.
- Actual result: rejected with MilestoneToApproveDoesNotExist. After the call,
  every milestone still has approval_count = 0 and approved_by = [].
- Was the operation atomic? YES. M0 and M1 were not partially approved.
- Error received: MilestoneError::MilestoneToApproveDoesNotExist  ->  Error(Contract, #6)
- Wallet balances of affected receivers: N/A (approve moves no funds).
- Escrow state after the call: milestones[0..3].approvals.approval_count = 0,
  approved_by empty for all.
- Transaction hash: N/A (contract-level test).
- Result: Pass
- Bug description: None.
```

```
Scenario B — Batch release with one unapproved milestone:
- Action executed: approve_milestones([0, 2]) (M1 left unapproved), then
  release_funds(milestone_index=[0, 1, 2], release_signer, trustless_work).
- Expected result: entire batch fails; no funds transferred for M0 or M2.
- Actual result: rejected with EscrowNotCompleted. No milestone released; all three
  receivers still hold 0; contract balance unchanged at the funded total; TW = 0; platform = 0.
- Was the operation atomic? YES. M0 and M2 were NOT released despite being approved.
- Error received: ReleaseError::EscrowNotCompleted  ->  Error(Contract, #7)
- Wallet balances of affected receivers (before -> after):
    M0 receiver: 0 -> 0
    M2 receiver: 0 -> 0
    contract:    30_000_000 -> 30_000_000 (unchanged)
- Escrow state after the call: milestones[0..3].released = false.
- Transaction hash: N/A (contract-level test).
- Result: Pass
- Bug description: None. The release validator checks every index up front
  (approval, dispute, resolved, released) before any auth or transfer.
```

```
Scenario C — Batch release with one already-released milestone:
- Action executed: approve_milestones([0, 1, 2]); release_funds([0]) individually;
  then release_funds(milestone_index=[0, 1, 2]) again (M0 already released).
- Expected result: the error on M0 prevents release of M1 and M2.
- Actual result: rejected with MilestoneAlreadyReleased. M1 and M2 remain unreleased
  and their receivers hold 0; M0's receiver balance is unchanged from the individual release.
- Was the operation atomic? YES. M1 and M2 were not released by the failed batch.
- Error received: ReleaseError::MilestoneAlreadyReleased  ->  Error(Contract, #8)
- Wallet balances of affected receivers (before -> after the failed batch):
    M0 receiver: 9_470_000 -> 9_470_000 (from the prior individual release; unchanged)
    M1 receiver: 0 -> 0
    M2 receiver: 0 -> 0
    contract:    20_000_000 -> 20_000_000 (only M0's 10_000_000 ever left)
- Escrow state after the call: M0.released = true, M1.released = false, M2.released = false.
- Transaction hash: N/A (contract-level test).
- Result: Pass
- Bug description: None.
```

```
Scenario D — Batch release of all milestones simultaneously:
- Action executed: 5 milestones with amounts 10/20/30/40/50 (x1e7 units), platform_fee 5%,
  all approved, then release_funds(milestone_index=[0, 1, 2, 3, 4]) in one call.
- Expected result: confirms in a single transaction; each receiver gets the correct net.
- Actual result: success. Per-milestone net amounts are exact and the contract drains to 0.
    M0: net 9_470_000   (tw 30_000,  platform 500_000)
    M1: net 18_940_000  (tw 60_000,  platform 1_000_000)
    M2: net 28_410_000  (tw 90_000,  platform 1_500_000)
    M3: net 37_880_000  (tw 120_000, platform 2_000_000)
    M4: net 47_350_000  (tw 150_000, platform 2_500_000)
    TW total 450_000 · platform total 7_500_000 · receivers total 142_050_000  (sums to 150_000_000)
- Was the operation atomic? YES (success path). All five released in one call.
- Error received: None.
- Wallet balances: each receiver credited exactly its net (see above); contract = 0 after.
- Escrow state after the call: milestones[0..5].released = true.
- Transaction hash: N/A (contract-level test).
- Result: Pass
- Note on Soroban resource limits: the test environment does not enforce on-chain CPU/mem
  metering, so this confirms FUNCTIONAL correctness (each receiver gets the right amount) but
  NOT that a 5-way release stays inside Soroban's resource budget on Testnet. That should be
  confirmed on-network. Context: approve_milestones is capped at MAX_BATCH_SIZE = 50 and
  dispute is capped at the milestone count, but release_funds has no explicit batch cap — it is
  bounded only by the milestone count (<= 50 at init), so a 50-way release is the realistic
  worst case to validate against the resource budget.
- Bug description: None at the contract level.
```

```
Scenario E — Empty batch:
- Action executed (three calls, each with milestone_index = []):
    approve_milestones([])
    release_funds([])
    dispute_milestones([])
- Expected result: a descriptive error each, NOT a silent no-op.
- Actual result: each returns a descriptive, distinct error. No state change occurs.
    approve_milestones([]) -> MilestoneError::BatchMilestoneApproveEmpty  -> Error(Contract, #9)
    release_funds([])      -> ReleaseError::ReleaseMilestonesEmpty        -> Error(Contract, #4)
    dispute_milestones([]) -> EscrowError::BatchMilestoneDisputeEmpty     -> Error(Contract, #42)
- Was the operation atomic? YES (trivially — no mutation; verified state unchanged).
- Error received: see the three codes above. (release_funds matches the issue's
  "ReleaseMilestonesEmpty" hint exactly.)
- Escrow state after the calls: no milestone approved, released, or disputed.
- Transaction hash: N/A (contract-level test).
- Result: Pass
- Bug description: None — none of the three silently succeed.
- Ordering nuance worth noting: release_funds checks the release-signer ROLE *before* the
  empty-batch check. So an empty array sent by a NON-release-signer returns
  OnlyReleaseSignerCanReleaseEarnings (#2), not ReleaseMilestonesEmpty (#4). From a valid
  release signer it returns ReleaseMilestonesEmpty. (approve and dispute check empty first.)
```

```
Scenario F — Batch dispute including an already-disputed milestone:
- Action executed: dispute_milestones([0]) individually, then
  dispute_milestones(milestone_index=[0, 1]) (M0 already in dispute).
- Expected result: entire batch fails atomically; M1 does NOT end up disputed.
- Actual result: rejected with MilestoneAlreadyDisputed. M1 is not disputed; M0 stays
  disputed; M2 untouched.
- Was the operation atomic? YES. M1 was not disputed despite being valid.
- Error received: EscrowError::MilestoneAlreadyDisputed  ->  Error(Contract, #43)
- Wallet balances of affected receivers: N/A (dispute moves no funds).
- Escrow state after the call: M0.dispute.is_disputed = true, M1.dispute.is_disputed = false,
  M2.dispute.is_disputed = false.
- Transaction hash: N/A (contract-level test).
- Result: Pass
- Bug description: None.
```

---

### Findings & answers to the issue's questions

- **No partial-failure bug exists at the contract level.** All six scenarios are fully
  atomic — in every failure case, *zero* milestones were mutated and *zero* funds moved.
  This is the critical finding the issue asked to surface, and it is the safe outcome.
- **Root cause of the atomicity guarantee:** every batch op (`approve_milestones`,
  `release_funds`, `dispute_milestones`) validates the whole index set first, mutates an
  in-memory copy, then writes once. The error always precedes the single `storage.set`, so
  there is no window in which some milestones are persisted and others are not. Soroban's
  transaction rollback backs this up.
- **Empty batches are handled explicitly** with three distinct, descriptive errors
  (`BatchMilestoneApproveEmpty`, `ReleaseMilestonesEmpty`, `BatchMilestoneDisputeEmpty`) —
  no silent no-op. One subtlety: `release_funds` validates the release-signer role before the
  empty check, so the exact code for an empty release depends on the caller's role.
- **Exact error codes** (each `#[contracterror]` has its own discriminant space):
  A `MilestoneToApproveDoesNotExist` #6 · B `EscrowNotCompleted` #7 ·
  C `MilestoneAlreadyReleased` #8 · E `BatchMilestoneApproveEmpty` #9 /
  `ReleaseMilestonesEmpty` #4 / `BatchMilestoneDisputeEmpty` #42 ·
  F `MilestoneAlreadyDisputed` #43.

### API-layer follow-ups (need the Testnet credentials)

These can't be answered by contract tests and are worth a quick live-API pass:

1. **Error mapping.** Confirm the API surfaces the codes above with accurate messages.
   TEST-01 already caught the API returning *"One of the selected milestones to approve does
   not exist"* for an over-cap platform fee — so don't assume the message matches the cause.
2. **Empty array over the wire.** Confirm `milestone_index: []` reaches the contract as an
   empty `Vec` (and thus returns the empty-batch error) rather than being dropped or rejected
   by request validation first.
3. **Scenario D resource budget.** Confirm a large real release (up to the 50-milestone
   bound) confirms on Testnet without hitting Soroban CPU/memory limits.
