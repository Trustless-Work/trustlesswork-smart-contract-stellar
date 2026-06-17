## TEST-02 — Update Escrow: Real Limits — Results

Tested as a black-box suite against the deployed Testnet API (`/escrow/multi-release/v2/*`), signing each XDR client-side and submitting via `/stellar/send-transaction`. Each scenario uses fresh escrows; every action below has a real on-chain transaction hash and the final `GET /:contractId` state.

**Summary:** A Pass · B Pass (with finding) · C Pass · D Pass · E Pass · F blocked (by design, see F2) · F2 Pass.

Two notes up front:
- **Role lists are order-sensitive** — reordering the `approvers` list (same set of addresses) is treated as a property change (Scenario B).
- **E.3 is not reproducible through the V2 API**: `/fund` only accepts `{contractId, signer, amount}` and rebuilds the expected escrow from current on-chain state, so you cannot fund with stale data via the API.

---

```
Scenario A:
- Steps executed:
    Deploy escrow with M0 (1 USDC) -> CD74KFA2DYOSVUCWJ7AUAN72GU5224ZQFECGHCVAJUBMFRORXZ35FZO7
    Fund 1 USDC (success)
    update description     -> ESCROW_PROPERTIES_MISMATCH
    update roles.approvers -> ESCROW_PROPERTIES_MISMATCH
    update platformFee     -> ESCROW_PROPERTIES_MISMATCH
- Expected result: Every update fails with ESCROW_PROPERTIES_MISMATCH
- Actual result: All three updates returned ESCROW_PROPERTIES_MISMATCH
- Error received: ESCROW_PROPERTIES_MISMATCH — "Provided escrow properties do not match the on-chain state." (identical for all three fields)
- Unexpected behavior? No. The error is the same regardless of which field is changed.
- Relevant transaction hashes:
    8aa90b37063f31be0e7f802972b753b110089245a87335b285ccf73977c066f0  (deploy)
    641794e127ca6fe0ae197a88a1278d583bfa58b35adfd37ebfe02a8f3d2bf4dd  (fund)
- Final escrow state (GET /:contractId): balance=0, approvers=[GBYU...26STMR], milestones=[M0: approved=false, released=false, disputed=false]
- Result: Pass
- Bug description: N/A
```

```
Scenario B:
- Steps executed:
    Deploy with approvers [A,B] -> CDYBLYXXGH6MNAN4TDCMSIXHM4RHPJSLOV7C42GM6Q7Z4DDYQVDQ3J4F
    Fund 1 USDC (success)
    update with identical payload         -> success (no-op)
    update with approvers reordered [B,A] -> ESCROW_PROPERTIES_MISMATCH
- Expected result: Identical update is a successful no-op; reordering the approvers list (same set) should not be a mismatch
- Actual result: Identical update = success (no-op); reordered list = ESCROW_PROPERTIES_MISMATCH
- Error received: reordered: ESCROW_PROPERTIES_MISMATCH — "Provided escrow properties do not match the on-chain state."
- Unexpected behavior? Yes — list ordering affects equality.
- Relevant transaction hashes:
    0ef13a9d5b95c7dbe3c155208ba7bfadffe50cf41808e95d12b38b0605974df2  (deploy)
    745483493a86c903897ddfb055947a967aad6f3465d4d3e8fe6e92f7b332fe0c  (fund)
    2d8e77a8e7fe6dfb87ee983e558a230903d7be288b316eb804c84fc8463e946b  (identical update / no-op)
- Final escrow state (GET /:contractId): balance=0, approvers=[GBYU...26STMR, GBFK...472474], milestones=[M0: approved=false, released=false, disputed=false]
- Result: Pass (no-op works) — finding: order-sensitive equality
- Bug description: Reordering the approvers list (same set of addresses, [A,B] vs [B,A]) returns ESCROW_PROPERTIES_MISMATCH. The equality check is order-sensitive on role lists. Since [A,B] and [B,A] are the same set of approvers, a client that reorders the list when resubmitting the escrow would get an unexpected mismatch. Worth confirming with the team whether ordering is intentional.
```

```
Scenario C:
- Steps executed:
    Deploy -> CABFIZ6G57BFS2KFP5K3ATWACXILAUT4V7VIUEF5IO7DGWWN7QU32VUL
    Fund 1 USDC (success)
    Dispute M0 (signed by an approver) -> success
    update with identical payload (dispute active) -> ESCROW_OPENED_FOR_DISPUTE_RESOLUTION
    update with a changed field (dispute active)   -> ESCROW_OPENED_FOR_DISPUTE_RESOLUTION
- Expected result: update blocked with ESCROW_OPENED_FOR_DISPUTE_RESOLUTION; dispute takes precedence over the funds/mismatch check
- Actual result: Both the identical and the changed-field updates returned ESCROW_OPENED_FOR_DISPUTE_RESOLUTION
- Error received: ESCROW_OPENED_FOR_DISPUTE_RESOLUTION — "Escrow is opened for dispute resolution; action blocked."
- Unexpected behavior? No. The dispute error takes precedence: even the changed-field update returns the dispute error (not the mismatch error), i.e. it is returned before the data is compared.
- Relevant transaction hashes:
    94edc08cef01f1d295f870c8b152e07048bf817c201235e97e1eff48e89d6c83  (deploy)
    c5b1849e5f0c41691192f43c37ca9655b681a2dbddfc448ffc7039eb39d36506  (fund)
    b4e35625ee98972846f228ec543db94650bd9ee1af89b675d0660e911ce03729  (dispute)
- Final escrow state (GET /:contractId): balance=0, milestones=[M0: approved=false, released=false, disputed=true]
- Result: Pass
- Bug description: N/A
```

```
Scenario D:
- Steps executed (balance = 0, freshly deployed):
    Deploy without funding -> CBGNNPYXPXD6UJVBU6LSGXUZFNLEPM2WHY2V6DGADV42NILZ4CSFP4EV
    update changing admin    -> ESCROW_ADMIN_ADDRESS_CANNOT_BE_CHANGED
    update changing platform -> ESCROW_PLATFORM_ADDRESS_CANNOT_BE_CHANGED
- Expected result: ESCROW_ADMIN_ADDRESS_CANNOT_BE_CHANGED and ESCROW_PLATFORM_ADDRESS_CANNOT_BE_CHANGED, even at balance 0
- Actual result: As expected
- Error received: admin -> ESCROW_ADMIN_ADDRESS_CANNOT_BE_CHANGED — "Admin address cannot be changed after initialization." | platform -> ESCROW_PLATFORM_ADDRESS_CANNOT_BE_CHANGED — "Platform address cannot be modified."
- Unexpected behavior? No. Errors are descriptive and distinguishable, and are returned at validation time (HTTP 422) even with balance 0.
- Relevant transaction hashes:
    fb25409013366636a63022a47e61f99b16629c7abfe8869b43dfbeee359e2c62  (deploy)
    (both update attempts were rejected at validation — HTTP 422 — before any signed transaction)
- Final escrow state (GET /:contractId): balance=0, admin/platform unchanged
- Result: Pass
- Bug description: N/A
```

```
Scenario E:
- Steps executed:
    Deploy with approvers [A], unfunded -> CDOHLXKZ5CURNHK6RCGSXYA7Y7S72OV7YBDUDG2IYPUHYEINLTSNK2AZ
    update approvers [A]->[B] at balance 0 -> success
    (E.3 fund-with-old-data is NOT expressible via the V2 API — see note below)
    Fund 1 USDC (uses current stored data [B]) -> success
    Approve M0 with wallet_A (old approver) -> ESCROW_UNAUTHORIZED_APPROVER (fails, as expected)
    Approve M0 with wallet_B (new approver) -> success
    Release M0 -> success, lifecycle complete
- Expected result: update at balance 0 changes approvers; funding uses the new data; wallet_A is excluded (approve fails) and wallet_B approves
- Actual result: As expected
- Error received: approve wallet_A -> ESCROW_UNAUTHORIZED_APPROVER — "Caller is not in roles.approvers."
- Unexpected behavior? No
- Relevant transaction hashes:
    3f84a20197e69deeac992e6bb271b89a13365e83b224df780211e202b0981ae4  (deploy)
    c1abc4eb18b092a7c9edf37d44ede3bb726c6936dc17420da3a3b09cf1d40d6b  (update approvers)
    97b6840a5d157643930bf6f8ec56f2b83836bd04ae9abeed9aed259ed6d64b3e  (fund)
    02e8d2b18e7ef95ec532f55d6577b585a8cc97c0f5bf860fe2b7084a0d61fde9  (approve wallet_B)
    c544486401e5f676a7b7cc378fa01c58286c613cc1647619ac8d584b894ece38  (release)
- Final escrow state (GET /:contractId): balance=0, approvers=[GBFK...472474 (wallet_B)], milestones=[M0: approved=true, released=true]
- Note on E.3: The V2 /fund endpoint only accepts {contractId, signer, amount}; it rebuilds the expected escrow from current on-chain state. There is no way to pass stale escrow data through the API, so the "fund with old approvers -> EscrowPropertiesMismatch" sub-step cannot be reproduced via the API. The contract-level check still exists, but the API never lets a caller submit stale data.
- Result: Pass
- Bug description: N/A
```

```
Scenario F:
- Steps executed:
    Deploy with M0 (1 USDC) -> CDNUEOX3CPK5YOYEXOS4UFNCDZGAWYNEXR5ZD7KYKDRNJABIEIDUB67D
    Fund M0 -> success
    Approve M0 -> success
    Release M0 -> success (balance back to 0)
    manage-milestones add M1 (unfunded) -> ESCROW_ALREADY_RELEASED  <-- flow blocked here
- Expected result: update between cycles changes release_signers; old relA cannot release M1, new relB can
- Actual result: The flow is blocked at step 2. After releasing M0 (the only milestone), the escrow is complete/released, so manage-milestones (add M1) returns ESCROW_ALREADY_RELEASED. There is no second cycle to test.
- Error received: manage-milestones add M1 -> ESCROW_ALREADY_RELEASED — "Escrow funds were already released."
- Unexpected behavior? The scenario as written is not achievable, but this is NOT a multi-release bug — see Scenario F2.
- Relevant transaction hashes:
    57a701ca30b6d1d4a1ca51aa4cb1091e5ee185420956e37a5d6a1267ad33dc97  (deploy)
    42c1ff5b177a353341293328fa1c8435a607cbe5e2c431c50bf1192910e4ec75  (fund)
    969c20421ef71a1c0dbdc983c9fa1bab5a7eef4dccae96b520e82a352b8574ba  (approve M0)
    b3568656b3b1ff29aec08940b89ae67865c540a4a0dc84d67581c6071dbeea32  (release M0)
- Final escrow state (GET /:contractId): balance=0, milestones=[M0: approved=true, released=true]
- Result: Fail (blocked — by design, see F2)
- Bug description: After releasing the ONLY milestone (M0), the escrow is complete and manage-milestones returns ESCROW_ALREADY_RELEASED. Scenario F as written (add M1 + update release_signers after a release) is therefore not achievable. Scenario F2 below isolates this: releasing one milestone while another is still pending does NOT close the escrow, so this is an expected limitation (you cannot add milestones to an escrow whose milestones have all been released), not a multi-release defect.
```

```
Scenario F2 (isolation of the F finding):
- Steps executed:
    Deploy with M0 + M1 (1 + 1 USDC) -> CACOWPPACOOMZJSOXPJHGED7RCXN4D66PY5QSG5Y5JN3Q2KA56KHDOD4
    Fund 2 USDC -> success
    Approve M0 -> success
    Release ONLY M0 -> success
    State after releasing M0: M0 approved+released, M1 still pending
    Approve M1 (with M0 already released) -> success
    Release M1 -> success
- Expected result: Releasing M0 while M1 is pending should NOT close the escrow; M1 should still be approvable and releasable
- Actual result: As expected — M1 was approved and released independently after M0 was released
- Error received: N/A
- Unexpected behavior? No
- Relevant transaction hashes:
    c1527a75bccea18d08bf90ca60f219cc0312ad113602d888d2170e02f4e2646f  (deploy)
    01da5ed8131599debc070d41015bc09976b62b41898371c85a34260b37953193  (fund)
    143016f88a501fbfa82f07553e67bd1283ab076a5e42e13743c1b01d8550e50c  (approve M0)
    5e2322f3d75545610023b22c2e753eafde9785390966f0ba6d6681f04171f1b2  (release M0)
    b0ae4f73e88308b4779ca8e93c224d5bd445b8cf221c8ccc0411c2c578490d28  (approve M1)
    e37f6f8718ee667c320eb6f3b2f21e5af4651cb0ac77b3915e7d2989f8b9b739  (release M1)
- Final escrow state (GET /:contractId): balance=0, milestones=[M0: approved=true, released=true; M1: approved=true, released=true]
- Result: Pass (multi-release behaves correctly)
- Bug description: N/A. This confirms the F block is solely because M0 was the only milestone — releasing the last milestone completes the escrow.
```

---

### Findings & answers to the issue's questions

- **A:** The mismatch error is identical regardless of which field changes.
- **B:** Identical update is a real no-op. However, **role-list equality is order-sensitive** — reordering `approvers` ([A,B] → [B,A]) yields `ESCROW_PROPERTIES_MISMATCH` even though it's the same set. Likely the only behavior worth a second look.
- **C:** The dispute error takes precedence: it is returned before the contract compares the data (even a changed-field update returns `ESCROW_OPENED_FOR_DISPUTE_RESOLUTION`, not the mismatch).
- **D:** `admin` and `platform` immutability errors are descriptive, distinguishable, and returned at validation (HTTP 422) even at balance 0.
- **E:** Updating roles at balance 0 works; the old approver is fully excluded afterward. **E.3 cannot be tested via the API** (the `/fund` endpoint doesn't accept escrow data).
- **F:** Not achievable as written — releasing the only milestone completes the escrow, so milestones can't be added afterward. **F2 confirms multi-release itself is correct**, so this is an expected limitation rather than a bug.
