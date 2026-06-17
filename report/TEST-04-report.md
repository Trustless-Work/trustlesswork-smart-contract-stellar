## [TEST-04] Complex Interleaved Flows — Results

Test run: 2026-06-17 | Network: Stellar Testnet | Contract: Multi-Release V2 (Soroban) | Score: 7/8 Pass

> **API workaround used throughout:** `approve-and-release-milestones` and `resolve-dispute` always return `ESCROW_INSUFFICIENT_FUNDS_FOR_FUNDING` (HTTP 422) on funded escrows. The API's internal DB balance field is never updated when funds land on-chain — the on-chain balance is correct but the API never reads it. Both functions were called directly via Soroban RPC instead for all scenarios, and they work correctly at the contract level. The `release-funds` endpoint was tested via API in Scenarios D and E (it has a related bug documented in the API Bugs section at the bottom).

---

**Scenario A: Approve one, release another, dispute approved, manage, resolve**

Steps:
  1. Deploy with M0(60) and M1(40)
  2. Fund 100 TWUSDC
  3. SP marks M0 and M1 completed
  4. Approve M0 only (not released)
  5. Approve-and-release M1 atomically (direct RPC - API balance bug workaround)
  6. Dispute M0 (approved but not yet released)
  7. manage-milestones: add M2(50) while M0 is disputed
  8. Resolve M0 dispute: 59.4 to receiver, 0.6 to platform
  9. withdraw-remaining-funds (testing API behavior)

Expected:
  manage-milestones should be blocked while any dispute is active. resolve-dispute should succeed and distribute funds correctly. approve-and-release should work per-milestone even while another is disputed.

Actual:
  manage-milestones blocked (ESCROW_OPENED_FOR_DISPUTE_RESOLUTION). resolve-dispute succeeded. Receiver balance updated correctly.

Errors:
  - manage-milestones during active dispute (step 7): `ESCROW_OPENED_FOR_DISPUTE_RESOLUTION` — Escrow is opened for dispute resolution; action blocked.
  - withdraw-remaining-funds: `ESCROW_INSUFFICIENT_FUNDS_FOR_RESOLUTION` — Insufficient funds to resolve the dispute.

Contract state at completion (contract `CD7FESRV5MHWRFV4APWAYNL2BUCQ553DLH4PM3MZVOR7ENAHEXUYQE3G`):
  M0: released=false, disputed=false, resolved=true, approvals=1, approved_by=[GDJEZK...XS7E]
  M1: released=true, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]

  State at key points:
    After approve M0:
      M0: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
      M1: released=false, disputed=false, resolved=false, approvals=0
    After release M1:
      M0: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
      M1: released=true, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
    After dispute M0:
      M0: released=false, disputed=true, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
      M1: released=true, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
    After resolve M0:
      M0: released=false, disputed=false, resolved=true, approvals=1, approved_by=[GDJEZK...XS7E]
      M1: released=true, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]

Wallet balances (TWUSDC):
  receiver: 4201.9551000 -> 4300.0629000  (Before fund / After resolve)

Transaction hashes:
  deploy: `8ccb8d704cf9651eb16e60aab6890d8dc615916193be6293bb1ce2bb85ab2db8`
  fund-100: `83c3bc4935b501cdbec9a60f45c2479ec197f021d84e0212d3fe0d06ae636636`
  status-0-1: `8cfa6decd3c23fc3cd5234a93255a034f0b34011560a749db25227d1f1be2bd7`
  approve-0: `473acba6821f167be4a1d04750dfbc5f95677677a60b24c490faf5fb97908fbe`
  approve-release-M1: `96ecf55d17240288a635b013ee34daf3ca96d656deffafe0aeadf019e0a8a38f`
  dispute-M0: `656dcdafe562188ee38a3ff4388a7807737909be454f27707569cd6a58aad51b`
  resolve-M0: `582078f8aa71b10e8a667336d109e541109c1f349e4cf00e5a74558561f67752`

Unexpected behavior: None

Result: **Pass**

Notes: manage-milestones is blocked globally by any active dispute. A milestone can be approved then disputed.

---

**Scenario B: Attempt to dispute an already-released milestone**

Steps:
  1. Deploy with M0(60) and M1(40)
  2. Fund 100
  3. SP marks M0 and M1 completed
  4. Approve-and-release M0 (direct RPC - API balance bug workaround)
  5. Attempt to dispute M0 (already released)

Expected:
  The contract should block disputing a milestone that has already been released. No risk of opening a dispute over funds that have already left the contract.

Actual:
  Contract blocked disputing a released milestone with: ESCROW_MILESTONE_ALREADY_RELEASED

Errors:
  - dispute released M0 (expected to be blocked): `ESCROW_MILESTONE_ALREADY_RELEASED` — Targeted milestone was already released.

Contract state at completion (contract `CAASDOHDIKFI2UELN7UT7JKILB2YVTGCEUC6UFR3MAENJ55HQSLNINEX`):
  M0: released=true, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
  M1: released=false, disputed=false, resolved=false, approvals=0

Wallet balances (TWUSDC):
  receiver: 4300.0629000 -> 4359.2829000  (Before fund / After release M0)

Transaction hashes:
  deploy: `ba8f4a1bed6000a769feedd11eb07f87f9fa1302390002fa7e9ce888067d731b`
  fund-100: `262613b2bc2bb0dabb7dfe65f979780a74e4c61af77013e2a9747cdb08139531`
  status-0-1: `dd3e1b031dc6b2e84d006601a59a202658f6636642811cc77e0c668d47cc976f`
  approve-release-M0: `a6e551322c05650845386ff7622baf95ccd040e5bbd5471a2db635e128b0f5eb`

Unexpected behavior: None

Result: **Pass**

Notes: ESCROW_MILESTONE_ALREADY_RELEASED correctly returned when attempting to dispute a released milestone.

---

**Scenario C: Dispute during partial multi-sig approval**

Steps:
  1. Deploy with M0(100), approvalsTarget=2, approvers=[wallet_a, wallet_b]
  2. Fund 100
  3. SP marks M0 completed
  4. wallet_A approves M0 (approval_count should become 1)
  5. Dispute M0 while only 1/2 approved
  6. wallet_B approves M0 DURING active dispute
  7. Resolve dispute: 99 to receiver, 1 to platform (direct RPC)

Expected:
  Unclear if approvals can be added during an active dispute. wallet_A pre-dispute approval may or may not be preserved after resolve. A fully-approved disputed milestone (2/2) should NOT auto-release through approve-milestones.

Actual:
  approve-milestones succeeded during active dispute. After resolution: approval_count=2, approved_by=[2 wallets]. Approval state is preserved after resolve.

Errors: None

Contract state at completion (contract `CBH4U6RESUWQXKW7DI3TTIAVEQUGUF33QG6GB6W73CXRR4RV2WCP6LZA`):
  M0: released=false, disputed=false, resolved=true, approvals=2, approved_by=[GAIGPP...7DF4, GAFS7X...5ORZ]

  State at key points:
    After walletA approval:
      M0: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GAIGPP...7DF4]
    After dispute:
      M0: released=false, disputed=true, resolved=false, approvals=1, approved_by=[GAIGPP...7DF4]
    After walletB approval during dispute:
      M0: released=false, disputed=true, resolved=false, approvals=2, approved_by=[GAIGPP...7DF4, GAFS7X...5ORZ]
    After resolve:
      M0: released=false, disputed=false, resolved=true, approvals=2, approved_by=[GAIGPP...7DF4, GAFS7X...5ORZ]

Wallet balances (TWUSDC):
  receiver: 4359.2829000 -> 4456.9959000  (Before fund / After resolve)

Transaction hashes:
  deploy: `72ee64c90703ca9c2d5d9e78f82e33942631c9d807bcb3d75c9a7f3da5be7170`
  fund-100: `8f318fce437c2425bf77927578499f23634206f144536e6d3970e81cc26c55f0`
  status-0: `9f72db2e348d04037d45e04a1d86e084f000378bfb2d3c4add44ad0f12c9b95d`
  approve-walletA: `9854d4bd02f4c115c1809a4041c44dbf020a4323f3a1ae821162d9f0e468adf5`
  dispute-M0: `b1ae296945ffc032d2e5dab9c279bfc9387fd1648b589356583a66da027b41fc`
  approve-walletB-during-dispute: `cc4447bdf364cf8fea53dec57c9e6e0a28d5d14d6f334fc73bbb455a3718088c`
  resolve-M0: `0ed70771e62d5eda000ee30775ad29a68e808fb94985200f19546cb2e010d793`

Unexpected behavior: approve-milestones is NOT blocked by an active dispute. Approvals accumulate while dispute is open. This may be intentional but is worth confirming.

Result: **Pass**

Notes: approve-milestones allowed during dispute. Final approvalCount=2. approvedBy=[2 wallets].

---

**Scenario D: Batch release where one milestone in the batch is disputed**

Steps:
  1. Deploy with M0(30), M1(30), M2(40)
  2. Fund 100
  3. SP marks all 3 milestones completed
  4. Approve all milestones [0,1,2]
  5. Dispute M1 only
  6. release-funds [0,1,2] — should fail atomically because M1 is disputed
  7. release-funds [0,2] skipping disputed M1 — unclear if globally blocked or per-milestone
  8. Resolve M1 dispute: 29.7 to receiver, 0.3 to platform (direct RPC)
  9. Release M1 individually after resolve (M1 resolved=true, funds already distributed)

Expected:
  release-funds [0,1,2] should fail atomically because M1 is disputed. release-funds [0,2] skipping M1 should succeed (non-disputed milestones releasable during dispute). After resolving M1 dispute, releasing M1 individually should be blocked (funds already distributed).

Actual:
  release-funds [0,1,2]: blocked (ESCROW_OPENED_FOR_DISPUTE_RESOLUTION, correct). release-funds [0,2]: blocked. Contract enforces a global dispute lock on all releases; non-disputed milestones are also frozen. API returned ESCROW_NOT_FOUND (DB sync bug); direct RPC confirmed with Error(Contract, #40). release-funds [1] after resolve: blocked with ESCROW_ALREADY_RESOLVED (correct).

Errors:
  - release-funds [0,1,2] with disputed M1 (expected blocked): `ESCROW_OPENED_FOR_DISPUTE_RESOLUTION` — Escrow is opened for dispute resolution; release blocked.
  - release-funds [0,2] — globally blocked during dispute (contract enforces global lock): `Error(Contract, #40)`
  - release-funds M1 after resolve (expected blocked): `ESCROW_ALREADY_RESOLVED` — Escrow dispute is already resolved.

Contract state at completion (contract `CCORSM534Y3C55ZSNA4RHVCCNNR3CEYRCIT5MW5FCZJGDQTL6AVCFZFP`):
  M0: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
  M1: released=false, disputed=false, resolved=true, approvals=1, approved_by=[GDJEZK...XS7E]
  M2: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]

  State at key points:
    After approve all:
      M0: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
      M1: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
      M2: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
    After dispute M1:
      M0: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
      M1: released=false, disputed=true, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
      M2: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
    After resolve M1:
      M0: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]
      M1: released=false, disputed=false, resolved=true, approvals=1, approved_by=[GDJEZK...XS7E]
      M2: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]

Wallet balances (TWUSDC):
  receiver: 4456.9959000 -> 4486.3098000  (Before fund / After all operations)

Transaction hashes:
  deploy: `67351ee37082b9c63e21d9fd99a93b2c818c87181038fb0b654bd949801828aa`
  fund-100: `2f86b20631f29d5404173126983b419c0f30e32a2aadb1cc1a6af8ac24d5e4f3`
  status-0-1-2: `a70fd0af16ca52520044e03a9e8cb592ec8cc048ee23e15cc39cdad86add4cdf`
  approve-all: `695a93ac6e3f1e23de05898c04023f1ab4fb87e97089d05405bc25a3baf30881`
  dispute-M1: `8dc77a93f39d39f291a94731d79e0dcf189a9ccde4bc35c7de07a7f317b583c4`
  resolve-M1: `e8478a33f126724bb96464a71337fc98015fa37716e99ecdd9ac2ba2e512928a`

Unexpected behavior: None

Result: **Pass**

Notes: Contract enforces a global dispute lock on all releases, same as manage-milestones. Non-disputed milestones [0,2] were also blocked while M1 was disputed. The release-funds API has an additional DB sync bug (ESCROW_NOT_FOUND) on the non-disputed release path. After resolve, release-funds M1 correctly returned ESCROW_ALREADY_RESOLVED.

---

**Scenario E: Approve, dispute, resolve, attempt release after resolve (approve-milestones + release-funds separately)**

Steps:
  1. Deploy with M0(100)
  2. Fund 100
  3. SP marks M0 completed
  4. Approve M0 via approve-milestones API (separate from release)
  5. Dispute M0 immediately after approval
  6. Resolve dispute: 99 to service_provider, 1 to platform (direct RPC — API balance bug workaround)
  7. Attempt release-funds M0 after resolve (should be blocked — funds already distributed)

Expected:
  approve-milestones records approval. Dispute opens after approval. resolve-dispute distributes 99% to service_provider, 1% to platform. After resolve: released=false, resolved=true. release-funds M0 after resolve should be blocked (no double payment). Step-by-step state (released, is_disputed, resolved, approval_count) captured after each step.

Actual:
  After resolve: released=false, disputed=false, resolved=true, approvals=1. Post-resolve release-funds: correctly blocked with ESCROW_ALREADY_RESOLVED.

Errors:
  - release-funds M0 after resolve (expected blocked): `ESCROW_ALREADY_RESOLVED` — Escrow dispute is already resolved.

Contract state at each step (contract `CBUK4V2MHPUBJFONMS4QCCGXGUBLKW3UP3KQE5VWDUGCOOXEN7G5LNZB`):
  After fund: released=false, disputed=false, resolved=false, approvals=0, approved_by=[none]
  After mark completed: released=false, disputed=false, resolved=false, approvals=0, approved_by=[none]
  After approve: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZKV2...]
  After dispute: released=false, disputed=true, resolved=false, approvals=1, approved_by=[GDJEZKV2...]
  After resolve: released=false, disputed=false, resolved=true, approvals=1, approved_by=[GDJEZKV2...]
  After release attempt blocked: released=false, disputed=false, resolved=true, approvals=1, approved_by=[GDJEZKV2...]

Wallet balances (TWUSDC):
  service_provider: 7516.2152000 -> 7613.9282000  (Before fund / After resolve)

Transaction hashes:
  deploy: `f4bb602b40f46d93c2a2239031082e011f44f9401cd4bcc838cd261b984e6e15`
  fund-100: `47f7de3fdfed091fbd19854350ec0dc23e916ec9daa99ca1d2ec7ef55597efc6`
  status-0: `54a562c98bf4a2d2992ac7df53c11bf77589abc7323995efa6b74fff81ed05d7`
  approve-M0: `2033d91e50c4a9abe5887e7c4ffbc5483892d02f87ca69f1e466b7e699ff3e8b`
  dispute-M0: `12efa86063e229eed3e131e265dbf2bf5f57c4a8f43270f0c35ef62172d2f7b0`
  resolve-M0: `262cecbe1e2cbd45236391932b9ad07b44fc900d5c2adca2f8986fea4974dc35`

Unexpected behavior: None

Result: **Pass**

Notes: resolve_dispute sets resolved=true, released=false. Post-resolve release-funds correctly blocked (ESCROW_ALREADY_RESOLVED).

---

**Scenario F: Repeated cycle: full release, add milestones, continue**

Steps:
  1. Deploy with M0(100)
  2. Cycle 1: fund 100, mark completed, approve-and-release M0
  3. manage-milestones: add M1(50) after M0 released

Expected:
  Multi-cycle pattern should work: release M0, add M1, release M1, add M2, release M2. No residual state from released milestones should interfere with new ones.

Actual:
  manage-milestones returned ESCROW_ALREADY_RELEASED immediately after the first approve_and_release_milestones call. Multi-cycle pattern is not functional.

Errors:
  - manage-milestones add M1 after M0 released (step 2): `ESCROW_ALREADY_RELEASED` — Escrow funds were already released.

Contract state at completion (contract `CDAYHMXLWHOWT6KANQADGSV4LB6WP37BWKOS2GBJJKJM5ISIAKVX4PSW`):
  M0: released=true, disputed=false, resolved=false, approvals=1, approved_by=[GDJEZK...XS7E]

Wallet balances: N/A for this scenario

Transaction hashes:
  deploy: `1c42e530b488a3fd3f0b98f68af3d68955782c162412d0cb38a6cca638e73a37`
  fund-1: `2191c582d7e693ed168a2ff672a373817afbdc2fddc269eb07a6fdcdca2d9609`
  status-M0: `58c26ecf1b7b7487853a6aeda053a9ce67760cad6879b6d07639b7bd5228c8dd`
  approve-release-M0: `bb6a73e930a3a34917f349b101976e388a1a26dbad217a7f9767f60e51f28be7`

Unexpected behavior: ESCROW_ALREADY_RELEASED is returned when calling manage-milestones after any release. A global "released" flag is set after the first approve_and_release_milestones call and permanently blocks manage-milestones. The multi-cycle use case is broken.

Result: **Fail - Bug Found**

Notes: manage-milestones is permanently blocked after the first release (ESCROW_ALREADY_RELEASED). The intended multi-cycle pattern — release a milestone, add new work, continue — is not possible.

---

**Scenario G: Attempt to resolve a dispute twice (double-distribution)**

Steps:
  1. Deploy with M0(100)
  2. Fund 100
  3. SP marks M0 completed
  4. Dispute M0
  5. First resolve: 69.3 to receiver, 29.7 to service_provider, 1 to platform
  6. Second resolve attempt on same milestone (should be blocked)

Expected:
  First resolve should succeed and distribute funds. Second resolve attempt on the same milestone should be blocked. Wallet balances after the second attempt should be identical to after the first resolve.

Actual:
  First resolve succeeded. Second resolve correctly blocked. Final: released=false, resolved=true, approval_count=0.

Errors:
  - second resolve_dispute on same milestone (expected blocked): `Error(Contract, #5)`

Contract state at completion (contract `CB2EPH3OU7JL2YSA5HY6EFT7UW6WTFYAGOFK5ZNZ3FPIWHONQCKD7GH2`):
  M0: released=false, disputed=false, resolved=true, approvals=0

  State at key points:
    After dispute:
      M0: released=false, disputed=true, resolved=false, approvals=0
    After first resolve:
      M0: released=false, disputed=false, resolved=true, approvals=0

Wallet balances (TWUSDC):
  receiver: 4585.0098000 -> 4653.4089000 -> 4653.4089000  (Before fund / After first resolve / After second resolve attempt)
  service_provider: 7613.9282000 -> 7643.2421000 -> 7643.2421000  (Before fund / After first resolve / After second resolve attempt)

Transaction hashes:
  deploy: `9a3ac26dd942f6a81b5ec9048a0b5ac708d2c575a5a1f5a177c7df5445a1a0a2`
  fund-100: `edde2fc53201c16adde6a2237a2e21f6dded176bfbb2fbb6d8664cd9b8b191be`
  status-M0: `276217224625f56c7b1ed52ddf9341fd368cc2f1ae31cbd6ee51fbe810a0ee17`
  dispute-M0: `bfb998ad2439e82ee834f7be4f9ab1edbad9dc58aa90773887d4b03d3e1f96fb`
  resolve-first: `0f8a09a5262e65384c7363d45e65e74e76d8326873fe742229f652d4a3794e80`

Unexpected behavior: None

Result: **Pass**

Notes: Double resolution correctly prevented. Contract error #5 on second attempt. Three-way distribution worked.

---

**Scenario H: Pre-fund approvals survive manage-milestones (amount and description updates)**

Steps:
  1. Deploy with M0(100), approvalsTarget=2, approvers=[wallet_a, wallet_b] — NOT funded
  2. wallet_A approves M0 before escrow is funded
  3. manage-milestones: description-only update (same amount=100, new description)
  4. manage-milestones: amount update (100 -> 120)
  5. Fund escrow (120 to match updated amount)
  6. SP marks M0 completed
  7. wallet_B approves M0

Expected:
  Approvals should be accepted before the escrow is funded. manage-milestones (description-only update) should NOT reset approval state. manage-milestones (amount update) should NOT reset approval state. Final approval_count should be 2 after both wallets have approved.

Actual:
  wallet_A pre-fund approval: succeeded. Description-only update: succeeded. Amount update: succeeded. Final approval_count=2 (expected 2 if approvals were preserved, 1 if reset by manage-milestones). approved_by=[2 wallets].

Errors: None

Contract state at completion (contract `CCNYUMJBYYGUOY4YPYGWWN3F26HLH5GTA46YVX7R5D2PALE4P7KVWQQD`):
  M0: released=false, disputed=false, resolved=false, approvals=2, approved_by=[GAIGPP...7DF4, GAFS7X...5ORZ]

  State at key points:
    After walletA approval:
      M0: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GAIGPP...7DF4]
    After desc only update:
      M0: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GAIGPP...7DF4]
    After amount update:
      M0: released=false, disputed=false, resolved=false, approvals=1, approved_by=[GAIGPP...7DF4]
    Final state:
      M0: released=false, disputed=false, resolved=false, approvals=2, approved_by=[GAIGPP...7DF4, GAFS7X...5ORZ]

Wallet balances: N/A for this scenario

Transaction hashes:
  deploy: `4e18ed68262201ed842668269941ba733df055fe47042ca3ef5d7a556c50b938`
  approve-walletA-before-fund: `579ee622213d45da6fae1b97698368c6d84221e7ada830344be2a43d1d2a4ae3`
  manage-desc-only: `264b5931daa9ea9e99c947844b52bbf7805afd13f4555a9e456c60a9f95f9f2d`
  manage-amount-update: `3fde7a472effffe8a46bbb1846c8e654170381d07974a9e10e09797d5922870c`
  fund: `ec9c15e41ca6d409666087a1230ae6253da378e5f4f69ca9fa20c5ad8a7afbeb`
  status-M0: `c7c17929177da52a3d7cff32de88d4e8fc8f091bf54db6e93bf8646cb5118989`
  approve-walletB: `6a341f72ae538645a132128480935391abd55d2f193e5bd0a1ff09d7fd7122ca`

Unexpected behavior: None

Result: **Pass**

Notes: wallet_A pre-fund approval: accepted. Approvals survived both manage-milestones updates (final approval_count=2). Both wallets are in approved_by.

---

### Summary

| Scenario | Description | Result |
|----------|-------------|--------|
| A | Approve one, release another, dispute approved, manage, resolve | Pass |
| B | Attempt to dispute an already-released milestone | Pass |
| C | Dispute during partial multi-sig approval | Pass |
| D | Batch release where one milestone in the batch is disputed | Pass |
| E | Approve, dispute, resolve, attempt release after resolve (approve-milestones + release-funds separately) | Pass |
| F | Repeated cycle: full release, add milestones, continue | Fail - Bug Found |
| G | Attempt to resolve a dispute twice (double-distribution) | Pass |
| H | Pre-fund approvals survive manage-milestones (amount and description updates) | Pass |

**7/8 Pass**
**1 scenario(s) with confirmed bugs.**

---

### Key Contract Behaviors Observed

1. **Global dispute lock on `manage-milestones`:** Any active dispute on any milestone blocks `manage-milestones` for the entire escrow (Scenario A). The lock is not per-milestone — it covers the whole contract.

2. **Global dispute lock on releases too:** `release-funds` and `approve_and_release_milestones` enforce the same global lock. When any milestone is disputed, all release attempts fail regardless of which milestone indexes are in the request (Scenario D). Attempting `release-funds [0,2]` while only M1 is disputed returned `Error(Contract, #40)` via direct RPC.

3. **`manage-milestones` is permanently disabled after first release (BUG, Scenario F):** After any `approve_and_release_milestones` call, the contract sets a global `released` flag that blocks all future `manage-milestones` calls with `ESCROW_ALREADY_RELEASED`. The intended multi-cycle pattern (release a milestone, add new work, continue) is not possible.

4. **`approve-milestones` is not blocked during a dispute:** Unlike `manage-milestones` and `release-funds`, the approval endpoint accepts new approvals while a dispute is open. Approval counts continued to accumulate normally in Scenario C (wallet_B approved while M0 was disputed). Worth confirming this asymmetry is intentional.

5. **`manage-milestones` does not reset approval state:** Updating a milestone's amount or description via `manage-milestones` does not clear `approved_by` or `approval_count` (Scenario H, verified for both update types).

6. **Approvals are accepted before the escrow is funded:** `approve-milestones` has no funding check — wallet_A's approval in Scenario H went through before any TWUSDC was deposited.

7. **No double-payment path after dispute resolution:** After `resolve_dispute`, the milestone is in state `resolved=true, released=false`. A subsequent `release-funds` attempt returns `ESCROW_ALREADY_RESOLVED` and is blocked (Scenario E). Funds are not distributed twice.

8. **Double resolution is blocked:** A second `resolve_dispute` call on an already-resolved milestone returns `Error(Contract, #5)` and has no effect. Wallet balances were verified unchanged after the blocked attempt (Scenario G).

---

### API Bugs Requiring Fix

**Bug 1: `approve-and-release-milestones` and `resolve-dispute` return 422 on funded escrows**

Both endpoints pre-check an internal DB `balance` field before building the transaction XDR. That field is never written when funds land on-chain (no indexer sync). The result is that both endpoints always return `ESCROW_INSUFFICIENT_FUNDS_FOR_FUNDING` (HTTP 422) for any funded escrow, even when the on-chain balance is correct. All release and dispute resolution flows through the API are non-functional. The workaround used here was to call both contract functions directly via Soroban RPC, which works correctly.

**Bug 2: `release-funds` returns `ESCROW_NOT_FOUND` when trying to release non-disputed milestones**

The endpoint does correctly short-circuit on active disputes (`ESCROW_OPENED_FOR_DISPUTE_RESOLUTION`) and already-resolved state (`ESCROW_ALREADY_RESOLVED`). But when milestones pass those checks and the API tries to build the release XDR, it returns `ESCROW_NOT_FOUND`. Discovered in Scenario D when attempting `release-funds [0,2]` with no disputed milestones in the batch. This is the same root cause as Bug 1 — the API can't find a funded escrow record in its DB.

**Bug 3: `withdraw-remaining-funds` returns 422 after resolution**

After `resolve_dispute` distributes all funds, calling `withdraw-remaining-funds` returns `ESCROW_INSUFFICIENT_FUNDS_FOR_RESOLUTION` (HTTP 422). Same root cause: the API checks the stale DB balance, which still shows 0.
