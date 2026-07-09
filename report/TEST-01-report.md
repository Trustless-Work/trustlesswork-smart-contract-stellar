## TEST-01 — Fee Calculation Accuracy — Results

Tested as a black-box suite against the deployed Testnet API (`/deployer/multi-release` + `/escrow/multi-release/*`), signing each XDR client-side and submitting via `/helper/send-transaction`. Every case uses a fresh escrow. The token is USDC at 7 decimals, so all amounts are checked to the smallest unit (1e-7). For each case I computed the expected split independently with an integer model that mirrors the contract math (`tw = floor(amount*30/10000)`, `platform = floor(amount*pf/10000)`, receiver gets the rest) and then compared it against the actual on-chain balance deltas.

**Summary:** Case 1 Pass · Case 2 Pass · Case 3 Pass (cap correct) **with a finding** · Case 4 Pass · Case 5 Pass · Case 6 Pass · Case 7 Pass.

Two notes up front:
- **`platformFee` is a percent at the API layer**, not basis points — sending `1` stores `100` bps on-chain. I verified this directly on a deploy before trusting any of the numbers below.
- **The TW fee address is a single global collector** (`GA6KH5VW...`); its balance accumulates across every escrow on the platform, so I measure its delta per release, not its absolute balance.

---

```
Case 1 — 100 USDC, platform_fee 1%:
- Input amount: 100 USDC
- platform_fee %: 1
- Expected service_provider: 98.7000000   Actual: 98.7000000
- Expected platform:          1.0000000   Actual:  1.0000000
- Expected TW fee:            0.3000000   Actual:  0.3000000
- Total discrepancy: 0
- Transaction hash (release): 8f01b89dd5660964087ab3fcb03422d8177ae880e20c4d11c6e56ac980159c79
- Contract: CATNNQUATG222GEDAFURZNPJSIEQOP3U36NOFE4CJ55C5Y5CUFIGCYCS
- Result: Pass
- Bug description: N/A — exact to the unit, sum back to 100.
```

```
Case 2 — 100 USDC, platform_fee 0%:
- Input amount: 100 USDC
- platform_fee %: 0
- Expected service_provider: 99.7000000   Actual: 99.7000000
- Expected platform:                  0   Actual:  0
- Expected TW fee:            0.3000000   Actual:  0.3000000
- Total discrepancy: 0
- Transaction hash (release): 26799abe6be1861d0b626e69a32ad9b53dde4eb6862e063709238b8dda5c33f8
- Contract: CAHVVW3KD3ZD4BF6NKQKJC5LQ3KMCIA3AYVG5D4EEIWNLMZ2ZA6JGTJH
- Result: Pass
- Bug description: N/A. A 0% fee does not throw a zero-amount transfer error — the platform
  transfer is simply skipped and the platform wallet ends up with exactly 0. Release succeeded.
```

```
Case 3 — platform_fee at the maximum:
- I probed the cap. Since the API takes a percent, I tested 98.5%, 99%, 99.01%, 100%
  (= 9850 / 9900 / 9901 / 10000 bps):
    98.50% (9850 bps) -> DEPLOYED   (CBRI2VQX4YELURJSN6NPYDZPAEKJIT5TUYM5BX3LA74JQAJZHXP3R724)
    99.00% (9900 bps) -> DEPLOYED   (CBFQMAERW36PRLYPV3ES5BBMIZG6SJT2CT4YV6ELMAJHT2FP357GRSQQ)
    99.01% (9901 bps) -> REJECTED
    100.0% (10000 bps) -> REJECTED
- Exact threshold: the max allowed platform_fee is 9900 bps (99%). 9901 and above are rejected.
  A fractional value below the cap (98.5%) deploys fine, so the rejection is purely the fee cap,
  not the fraction.
- Is the error descriptive? No — this is the finding. An over-cap deploy returns:
    HTTP 400 {"message": "One of the selected milestones to approve does not exist"}
  which has nothing to do with the actual cause. It should say the platform fee is too high.
- Result: Pass on the numeric cap (9900 bps enforced correctly); the error message is wrong.
- Bug description: Over-cap deploys fail with a misleading "milestone does not exist" message
  instead of a fee-too-high / PlatformFeeTooHigh error. The behaviour is correct, the reporting is not.
```

```
Case 4 — 33 USDC, platform_fee 3% (the "fractional" 0.99 case):
- Input amount: 33 USDC
- platform_fee %: 3
- Expected service_provider: 31.9110000   Actual: 31.9110000
- Expected platform:          0.9900000   Actual:  0.9900000
- Expected TW fee:            0.0990000   Actual:  0.0990000
- Total discrepancy: 0
- Transaction hash (release): 9cce494f13ee28e49a6ef5a65f8bcbb206b409a45b40ef2be7fdec4a87bb1260
- Contract: CBK3AZ4MZSJRPZSL2OE2RIRAH56T4QQFDJIFQ5WM22J5DVITHH3GQSTB
- Result: Pass
- Bug description: N/A. The "non-whole" 0.99 USDC is actually exact at 7 decimals (9,900,000 units),
  so there is no rounding difference for anyone to absorb. The service_provider receives exactly the
  nominal amount — not more, not less. (Where a true sub-unit remainder did exist, it's the receiver
  that would absorb it and get slightly more, never less, since fees are floored.)
```

```
Case 5 — milestones 10 / 20 / 70 USDC, platform_fee 5%, released separately:
- Released each milestone in its own transaction, then summed everything received.
- Per-party totals across the three releases:
    Expected service_provider: 94.7000000   Actual: 94.7000000
    Expected platform:          5.0000000   Actual:  5.0000000
    Expected TW fee:            0.3000000   Actual:  0.3000000
- Sum received by all parties: 100.0000000  (must equal 100)
- Total discrepancy: 0 — nothing lost, nothing created.
- Transaction hashes (release M0/M1/M2):
    97fef670569574a3f8e51497e781059811348a8caa5137f523a4a5713aed7720
    575193854e578b62c3f17915da02cd618dd8421ae24467059a9443b4c3789e0a
    bacdb17a9db27206b552903ef7f8aab094999657238b33d868ab491b71b0bb56
- Contract: CC64JSUDMBC7YBREWHZWKUQWIADXZTDMJNBF5THJRTBZ4MQ3TNILCFST
- Result: Pass
- Bug description: N/A. Fees are computed per milestone on each milestone's own amount; the totals reconcile to 100 exactly.
```

```
Case 6 — dispute resolution, 100 USDC, 70 client / 30 service_provider:
- platform_fee: 5% (the issue didn't specify one for this case; I used 5% so the fees are visible).
- Disputed M0 (signed by the service_provider, since the dispute resolver can't raise its own dispute),
  then resolved with the 70/30 split.
- Expected client net:           66.2900000   Actual: 66.2900000
- Expected service_provider net: 28.4100000   Actual: 28.4100000
- Expected platform fee:          5.0000000   Actual:  5.0000000
- Expected TW fee:                0.3000000   Actual:  0.3000000
- Sum: 100.0000000   Total discrepancy: 0
- Transaction hash (resolve): 78053e08d8f9e7ba26727753cd68666d0e495a63d78527203c4a304127e36ddb
- Contract: CC7UK7XUT4AM6YGKRDP7BHIJ5IJFQOWBIPOVHXWLVPWBNLTZAD3YYF44
- Result: Pass
- Answer to the question: the fees are NOT taken proportionally out of each party separately and they
  are NOT ignored either — they come off the total first. TW and platform get the exact global fee on
  the 100, then the remaining 94.7 is split pro-rata (70/30) between client and service_provider. So
  each party effectively bears its share of the fee. Any sub-unit remainder would go to the last
  recipient by address order; here the split is exact so there's no remainder.
```

```
Case 7 — fee on withdraw-remaining-funds:
- Built a residual on purpose: funded 100, disputed M0, resolved distributing only 50 to the
  service_provider, which leaves 50 sitting in the contract. Then called withdraw-remaining-funds
  distributing the 50 to the client.
- Is a fee charged on the residual? Yes:
    on the 50 withdrawn -> client net 47.3500000, platform 2.5000000, TW 0.1500000
    (47.35 + 2.5 + 0.15 = 50.0 — same standard fee model, exact)
- Contract balance after the withdraw: 0 (reaches exactly zero, confirmed via GET by contractId)
- Total discrepancy: 0
- Transaction hash (withdraw): 52250b3849fd146e242f217a02d3edf6ae0e5cbbafafafe9082fe03f67a5c702
- Contract: CDQHJZ6CPPG4QNBMBCDLQ2IRU5C6EJZHJ2T2BHMBANYGB33VK2LIE7TS
- Result: Pass
- Bug description: N/A. The withdraw path charges the same fee as a normal distribution and the
  contract zeroes out cleanly.
```

---

### Findings & answers to the issue's questions

- **The fee math is exact to the single token unit** in every case — standard release, multiple
  milestones released separately, dispute resolution, and withdraw-remaining. Total discrepancy was
  0 everywhere, and funds always summed back to the input with nothing lost or created.
- **Case 3 is the one thing worth fixing:** the 99% (9900 bps) cap is enforced correctly, but a deploy
  that exceeds it fails with `"One of the selected milestones to approve does not exist"` — a
  misleading message that points integrators at the wrong problem. It should report a fee-too-high error.
- **Case 4's premise doesn't bite at 7 decimals:** 0.99 USDC is exact, so there's no rounding for
  anyone to absorb. When a real sub-unit remainder does occur, the receiver absorbs it (gets slightly
  more), because both fees are floored.
- **0% platform fee is safe** (Case 2) — no zero-amount transfer error, platform just gets 0.
- **In a dispute (Case 6), fees come off the total first**, then the net is split pro-rata; TW and
  platform receive the exact global fee.
