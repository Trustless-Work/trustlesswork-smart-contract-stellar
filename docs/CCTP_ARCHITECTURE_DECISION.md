# ADR-001: Cross-Chain CCTP Receiver Placement (Issue #91)

**Status:** Accepted  
**Date:** 2026-06-20  
**Branch:** `single-release-develop`  
**Issue:** [#91 — Cross-Chain USDC Release via CCTP](https://github.com/grantfox-oss/trustlesswork-smart-contract-stellar/issues/91)

---

## Context

Issue #91 specifies `cross_chain_receiver: Option<CrossChainReceiver>` on the **Milestone** struct, alongside per-milestone `amount`, `receiver`, and `released` fields.

The `single-release-develop` branch uses a different model:

| Concern | Current implementation |
|---------|------------------------|
| Release trigger | All milestones approved → single `release_funds` call |
| Payout amount | `escrow.amount` (escrow-level) |
| Stellar recipient | `escrow.roles.receiver` (escrow-level) |
| Milestone struct | `description`, `status`, `evidence`, `approved` only |

Adding per-milestone amount/receiver fields would be a separate multi-release migration, not scoped to CCTP alone.

---

## Decision

**Place `cross_chain_receiver: CrossChainReceiver` on the `Escrow` struct**, not on `Milestone`.

| Field | Role |
|-------|------|
| `escrow.cross_chain_receiver` | When configured (domain ≠ `CROSS_CHAIN_DISABLED_DOMAIN`), release burns USDC via CCTP |
| `escrow.roles.receiver` | Stellar address for standard release; also receives 7th-decimal remainder on cross-chain release |
| `escrow.receiver_memo` | Unchanged; not used in release today; out of scope for CCTP v1 |

### Soroban storage note

`Option<CrossChainReceiver>` is **not supported** inside `#[contracttype]` structs in Soroban SDK 26 (nested custom types in `Option` fail `ScVal` conversion). We use a **sentinel pattern** instead:

- `CROSS_CHAIN_DISABLED_DOMAIN` (`u32::MAX`) = standard Stellar release (backwards compatible)
- `default_cross_chain_receiver(env)` = default for new escrows
- `is_cross_chain_configured(&receiver)` = checks whether CCTP path applies

When the domain is the sentinel value, release behavior is **identical to the current contract**.

---

## `deposit_for_burn` Interface (Circle TokenMessengerMinter)

Issue #91 shows a simplified 4-argument call. Circle's Stellar `TokenMessengerMinter` exposes:

```rust
fn deposit_for_burn(
    e: &Env,
    caller: Address,              // escrow contract address (requires auth)
    amount: i128,                 // local 7-decimal USDC amount
    destination_domain: u32,
    mint_recipient: BytesN<32>,
    burn_token: Address,          // Stellar USDC SAC address
    destination_caller: BytesN<32>, // zero = any relayer may submit attestation
    max_fee: i128,                // destination-chain fee budget (local decimals)
    min_finality_threshold: u32,  // 2000 = standard finality
);
```

**Auth / approval:** The escrow contract must call as `caller` with `require_auth`, and must `token.approve` the TokenMessenger for the burn amount before invoking (per Circle test utilities).

**Decimal handling:** TokenMessenger normalizes 7→6 decimals internally during burn. The escrow contract will still explicitly transfer any remainder stroops to `roles.receiver` so funds are never locked (issue requirement).

**Outbound only:** Use `TokenMessengerMinter` directly. `CctpForwarder` is for **inbound** Stellar transfers only.

---

## Valid Destination Domains

| Chain | Domain |
|-------|--------|
| Ethereum | 0 |
| Avalanche | 1 |
| OP Mainnet | 2 |
| Arbitrum One | 3 |
| Solana | 5 |
| Base | 6 |
| Polygon PoS | 7 |

Reject: unknown domains, domain `27` (Stellar self), zero-byte `recipient`.

---

## Out of Scope (v1)

- Per-milestone cross-chain receivers (multi-release branch)
- Cross-chain payout in dispute resolution / `withdraw_remaining_funds`
- Mainnet address configuration (testnet constants only for now)
- Wiring `receiver_memo` into CCTP hook data

---

## Consequences

- **Positive:** Minimal storage change; backwards compatible; matches single-release release flow
- **Positive:** No milestone struct migration
- **Trade-off:** One cross-chain destination per escrow, not per milestone (acceptable for single-release)
- **PR note:** Document this deviation from issue #91's Milestone-level field in the PR description
