## TEST-03 — Batch Operations: Atomicity and Partial Failure Behavior — Results

Tested two ways, and they agree:

1. **Black-box against the deployed Testnet API** (`/escrow/multi-release/v2/*`),
   authenticating with `x-api-key`, signing each returned `unsignedXdr` client-side, and
   submitting via `/stellar/send-transaction`. Every scenario uses a **fresh escrow**. The
   token is **native XLM** (testnet SAC `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`,
   7 decimals), chosen so no classic trustlines are needed; amounts are checked to the
   smallest unit (stroop). After each failing call I re-read the escrow
   (`GET /escrow/multi-release/v2/{contractId}`) and the affected receivers' on-chain
   balances to confirm nothing was partially applied.
2. **Reproducible contract-level Rust tests** — `contracts/escrow/src/tests/batch_atomicity.rs`
   (run with `cargo test -p escrow batch_atomicity::`). These assert the exact
   `#[contracterror]` and read state/balances back deterministically. They are the
   complementary, re-runnable proof behind the on-chain runs below.

**Summary:** A Pass · B Pass · C Pass · D Pass · E Pass · F Pass.
**Every scenario was atomic. No case applied partial changes before failing.**

Notes up front:
- **`platformFee` is a percent at the API layer** (sending `5` = 5% = 500 bps on-chain),
  consistent with TEST-01.
- The batch ops return a descriptive error and roll back entirely; the contract validates
  the *whole* index set before mutating, then writes once, so there is no partial-write
  window.
- **Scenario E finding:** an empty `milestoneIndexes` is rejected by the API's request
  validation (`HTTP 400`, before the contract is reached), not by the contract's own
  empty-batch errors — see Scenario E.

All transaction hashes are on Testnet; links go to stellar.expert.

---

```
Scenario A — Batch approve with an out-of-range index:
- Action executed: POST /escrow/multi-release/v2/approve-milestones
    { contractId, approver, milestoneIndexes: [0, 1, 99] }
  on a funded 3-milestone escrow (M0, M1, M2), each amount 10 XLM, approvalsTarget 1.
- Expected result: whole batch rejected; M0 and M1 NOT approved.
- Actual result: rejected at build/simulate. M0, M1, M2 all still approvalCount = 0.
- Was the operation atomic? YES.
- Error received: HTTP 404  ESCROW_MILESTONE_TO_APPROVE_DOES_NOT_EXIST
    "Milestone to approve does not exist."
- Wallet balances of affected receivers: N/A (approve moves no funds).
- Escrow state after the call: milestones[0..3].approvals.approvalCount = 0.
- Contract: CDUVSJX4HOR3GMVW5PTCZQKSVFTWZYML3V376O6E5RDWEMHFH22VGCNI
- Transaction hashes:
    deploy 02cb959a8f200fd1b14b4382c328657c6445c1f5eb6848e35076ff3cd05517fc
    fund   57ca61fd0056163968ef49f68a5fa8a499dba4b01fba5717365ce2c79ae67768
    (the approve call never produced a transaction — rejected before signing)
- Result: Pass
- Bug description: None.
```

```
Scenario B — Batch release with one unapproved milestone:
- Action executed: approve-milestones [0, 2] (M1 left unapproved), then
    POST /escrow/multi-release/v2/release-funds { contractId, releaseSigner, milestoneIndexes: [0,1,2] }.
- Expected result: entire batch fails; no funds transferred for M0 or M2.
- Actual result: rejected. No milestone released; all three receiver balances unchanged.
- Was the operation atomic? YES. M0 and M2 were NOT released despite being approved.
- Error received: HTTP 409  ESCROW_NOT_COMPLETED
    "Targeted milestone has pending approvals; release cannot proceed."
- Wallet balances of affected receivers (delta across the failed call): M0 = 0, M1 = 0, M2 = 0.
- Escrow state after the call: released = [false, false, false];
    approvalCount = [1, 0, 1] (M0/M2 approved, M1 not).
- Contract: CDEVWCZMQTNBY2A26AIA6WYUNZYJXEE2FNBBNYWSNLBO6C4QK2BARWDF
- Transaction hash: none (release rejected before signing).
- Result: Pass
- Bug description: None.
```

```
Scenario C — Batch release with one already-released milestone:
- Action executed: approve-milestones [0,1,2]; release-funds [0] (individual, succeeds);
    then release-funds [0,1,2] (M0 already released).
- Expected result: the error on M0 prevents release of M1 and M2.
- Actual result: rejected. M1 and M2 remain unreleased; their receivers received nothing;
    M0's receiver balance unchanged by the failed batch.
- Was the operation atomic? YES.
- Error received: HTTP 409  ESCROW_MILESTONE_ALREADY_RELEASED
    "Targeted milestone was already released."
- Wallet balances of affected receivers (delta across the failed batch): M0 = 0, M1 = 0, M2 = 0.
- Escrow state after the call: released = [true, false, false].
- Contract: CAKZII5JBV3LJPWE56R5376U7YK2BTVDAZ7FWQ3LWJKHKDOVTCFG5IFZ
- Transaction hashes:
    release M0 (individual) eec9084301b6036b2d1767ad761e904b4018900e39bcaf33b7f9620392dcac70
    (the batch release never produced a transaction — rejected before signing)
- Result: Pass
- Bug description: None.
```

```
Scenario D — Batch release of all milestones simultaneously:
- Action executed: 5 milestones amounts 10 / 20 / 30 / 40 / 50 XLM, platformFee 5%, all
    approved, then POST /escrow/multi-release/v2/release-funds { milestoneIndexes: [0,1,2,3,4] }.
- Expected result: confirms in a single transaction, no Soroban resource-limit error,
    each receiver gets the correct net.
- Actual result: confirmed (HTTP 200, no resource error). Receiver balance deltas (stroops):
    M0  94,700,000   (9.47 XLM)   expected 94,700,000
    M1 189,400,000  (18.94 XLM)   expected 189,400,000
    M2 284,100,000  (28.41 XLM)   expected 284,100,000
    M3 378,800,000  (37.88 XLM)   expected 378,800,000
    M4 473,500,000  (47.35 XLM)   expected 473,500,000
  Platform fee delta 75,000,000 (7.5 XLM) = expected. Every milestone released.
  Fee model per milestone: tw = amount*30/10000 (0.3%), platform = amount*5/100 (5%).
- Was the operation atomic? YES (success path); all five released in one transaction.
- Error received: None.
- Escrow state after the call: released = [true, true, true, true, true].
- Contract: CC3PQRZRPCF2GGAJVJIKYUN3ACON7O6NHOOHWPMD7GMUPWGNB3ZXGC53
- Transaction hash (release):
    c4ede26a9fd2b4761ca8b4173b03164b0ff9863b4416f7504f47f2367877f946
- Result: Pass
- Bug description: None. A 5-way release stays within Soroban resource limits on Testnet and
  every receiver is paid to the exact stroop.
```

```
Scenario E — Empty batch:
- Action executed (three calls, each with milestoneIndexes = []):
    POST approve-milestones [], POST release-funds [], POST dispute-milestones [].
- Expected result: a descriptive error each, NOT a silent no-op.
- Actual result: all three return HTTP 400 BAD_REQUEST
    "milestoneIndexes must contain at least 1 elements". Escrow state unchanged.
- Was the operation atomic? YES (trivially — no mutation; verified state unchanged).
- Error received: HTTP 400 BAD_REQUEST (request-validation layer) for all three endpoints.
- Escrow state after the calls: no milestone approved, released, or disputed.
- Contract: CBL4TESRQAFKFSHOIA45UONERZK4REB346IKE3R5NJ4SUAYR3H6BMPN3
- Transaction hash: none (rejected before signing).
- Result: Pass
- Finding: the empty batch is caught by the API's DTO validation BEFORE the contract runs,
  so the contract's own dedicated errors (ReleaseMilestonesEmpty / BatchMilestoneApproveEmpty /
  BatchMilestoneDisputeEmpty — confirmed reachable in the Rust tests) are never surfaced over
  the API. The outcome the issue asked about is still correct: a descriptive error, never a
  silent success. Only the SOURCE of the error differs (API validation vs. contract).
```

```
Scenario F — Batch dispute including an already-disputed milestone:
- Action executed: dispute-milestones [0] (individual, succeeds), then
    POST /escrow/multi-release/v2/dispute-milestones { signer, milestoneIndexes: [0,1], reason }.
- Expected result: entire batch fails atomically; M1 does NOT end up disputed.
- Actual result: rejected. M1 is not disputed; M0 stays disputed.
- Was the operation atomic? YES.
- Error received: HTTP 409  ESCROW_MILESTONE_ALREADY_DISPUTED
    "Targeted milestone is already in dispute."
- Wallet balances of affected receivers: N/A (dispute moves no funds).
- Escrow state after the call: dispute.isDisputed = [true, false, false].
- Contract: CBZWTZKZXR4NM4PSTHT7MEBSD45KRFHSWWY52WRNIWYOAHDX4E6OHNQQ
- Transaction hashes:
    dispute M0 (individual) ab257a4ddde1aa282a183ddb2c9d87450b1a4f56a5eb69bda2188e49d692734e
    (the batch dispute never produced a transaction — rejected before signing)
- Result: Pass
- Bug description: None.
```

---

### Findings & answers to the issue's questions

- **No partial-failure bug.** All six scenarios are fully atomic — in every failure case
  *zero* milestones were mutated and *zero* funds moved, confirmed both on-chain (state +
  balance deltas) and in the contract-level Rust tests. This is the critical question the
  issue raised, and the result is the safe one.
- **Why it's atomic:** each batch entry point (`approve_milestones`, `release_funds`,
  `dispute_milestones`) validates the entire index set first, mutates an in-memory copy, then
  commits with a single `storage.set`. The error returns before that write, so there is no
  partial-write window; Soroban's transaction rollback backs it up.
- **Error reporting is accurate** for these batch ops — the API maps contract errors to
  specific, correct codes: `ESCROW_MILESTONE_TO_APPROVE_DOES_NOT_EXIST` (404),
  `ESCROW_NOT_COMPLETED` (409), `ESCROW_MILESTONE_ALREADY_RELEASED` (409),
  `ESCROW_MILESTONE_ALREADY_DISPUTED` (409). (Unlike the misleading message TEST-01 found on
  the fee-cap path.)
- **Empty batches (Scenario E)** never silently succeed. They are rejected at the API
  request-validation layer with `HTTP 400 "milestoneIndexes must contain at least 1
  elements"`. Note this shadows the contract's own empty-batch errors, which the Rust tests
  confirm are otherwise reachable (`ReleaseMilestonesEmpty` #4, `BatchMilestoneApproveEmpty`
  #9, `BatchMilestoneDisputeEmpty` #42).
- **Scenario D** confirms a full 5-way release confirms in one transaction within Soroban's
  resource budget, with each receiver paid the exact net amount.

### Reproducibility

- Contract-level: `cargo test -p escrow batch_atomicity::` (deterministic, no network/keys).
- On-chain: a small Node harness builds each operation against the Testnet API, signs the
  returned XDR with `@stellar/stellar-sdk`, and submits via `/stellar/send-transaction`. The
  contract IDs and transaction hashes above are verifiable on
  `https://stellar.expert/explorer/testnet`.
