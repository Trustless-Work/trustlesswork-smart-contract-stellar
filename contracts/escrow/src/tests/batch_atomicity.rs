//! Batch operation atomicity tests — issue #87 ([TEST-03]).
//!
//! V2 introduces batch milestone operations (`approve_milestones`,
//! `release_funds`, `dispute_milestones`) that take a `Vec<u32>` of milestone
//! indices. The contract MUST treat each batch as all-or-nothing: if any index
//! in the batch is invalid or in the wrong state, NO milestone in that call may
//! be mutated and NO funds may move.
//!
//! These tests deploy a fresh escrow per scenario, trigger the failure
//! condition, assert the exact error code, and then read the escrow state and
//! token balances back to prove that nothing was partially applied.
//!
//! Why atomicity holds in this contract: every batch entry point first runs a
//! `validate_batch_*` pass over the WHOLE index set, then mutates an in-memory
//! copy, then writes it back with a single `storage.set`. Any error returns
//! before that single write, so partial application is structurally impossible
//! — and Soroban's transaction-level rollback is a second line of defense.

extern crate std;

use crate::contract::EscrowContractClient;
use crate::error::{EscrowError, MilestoneError, ReleaseError};
use crate::storage::types::{Dispute, Escrow, Milestone, MilestoneApprovals, Roles, Trustline};
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    vec, Address, Env, String, Vec,
};

use super::helpers::{create_escrow_contract, create_usdc_token};

const TRUSTLESS_WORK_FEE_BPS: i128 = 30;
const BASIS_POINTS_DENOMINATOR: i128 = 10_000;

struct Setup<'a> {
    client: EscrowContractClient<'a>,
    messenger: Address,
    token: TokenClient<'a>,
    token_admin: StellarAssetClient<'a>,
    approver: Address,
    release_signer: Address,
    platform: Address,
    trustless_work: Address,
    funder: Address,
    receivers: Vec<Address>,
    escrow: Escrow,
}

/// Builds and initializes a fresh escrow with one milestone per entry in
/// `amounts`. Every milestone starts with status "Completed", the given
/// approval `target`, and its own distinct receiver so per-milestone fund flow
/// can be checked independently.
fn build_escrow<'a>(env: &'a Env, amounts: &[i128], target: u32, platform_fee: u32) -> Setup<'a> {
    let token_issuer = Address::generate(env);
    let escrow_admin = Address::generate(env);
    let approver = Address::generate(env);
    let service_provider = Address::generate(env);
    let release_signer = Address::generate(env);
    let dispute_resolver = Address::generate(env);
    let platform = Address::generate(env);
    let trustless_work = Address::generate(env);
    let funder = Address::generate(env);

    let (token, token_admin) = create_usdc_token(env, &token_issuer);

    let mut milestones = Vec::new(env);
    let mut receivers = Vec::new(env);
    for amount in amounts.iter() {
        let receiver = Address::generate(env);
        receivers.push_back(receiver.clone());
        milestones.push_back(Milestone {
            description: String::from_str(env, "Milestone"),
            status: String::from_str(env, "Completed"),
            evidence: String::from_str(env, "Evidence"),
            approvals: MilestoneApprovals {
                target,
                approval_count: 0,
                approved_by: vec![env],
            },
            amount: *amount,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(env, ""),
                resolved: false,
            },
            released: false,
            receiver: crate::tests::helpers::test_receiver(env, &receiver),
        });
    }

    let roles = Roles {
        approvers: vec![env, approver.clone()],
        service_providers: vec![env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![env, release_signer.clone()],
        dispute_resolvers: vec![env, dispute_resolver.clone()],
        admin: escrow_admin.clone(),
        observers: vec![env],
    };

    let escrow = Escrow {
        engagement_id: String::from_str(env, "batch_atomicity"),
        title: String::from_str(env, "Batch Atomicity Escrow"),
        description: String::from_str(env, "Issue #87 batch atomicity tests"),
        roles,
        platform_fee,
        milestones,
        trustline: Trustline {
            address: token.address.clone(),
        },
        receiver_memo: 0,
    };

    let messenger = crate::tests::helpers::register_mock_token_messenger(env);
    let client = create_escrow_contract(env, &escrow_admin).client;
    client.initialize_escrow(&escrow);

    Setup {
        client,
        messenger,
        token,
        token_admin,
        approver,
        release_signer,
        platform,
        trustless_work,
        funder,
        receivers,
        escrow,
    }
}

/// Funds the escrow contract with `total` via the real `fund_escrow` path.
/// Must be called before any approve/dispute mutates the stored escrow, because
/// `fund_escrow` validates the expected escrow equals the stored one.
fn fund(s: &Setup, total: i128) {
    s.token_admin.mint(&s.funder, &total);
    s.client.fund_escrow(&s.funder, &s.escrow, &total);
}

fn receiver(s: &Setup, i: u32) -> Address {
    s.receivers.get(i).unwrap()
}

// ---------------------------------------------------------------------------
// Scenario A: Batch approve with an out-of-range index [0, 1, 99].
// ---------------------------------------------------------------------------
#[test]
fn scenario_a_approve_out_of_range_index_is_atomic() {
    let env = Env::default();
    let s = build_escrow(&env, &[10_000_000, 10_000_000, 10_000_000], 1, 500);
    fund(&s, 30_000_000);

    let result = s
        .client
        .try_approve_milestones(&vec![&env, 0u32, 1u32, 99u32], &s.approver);

    // The whole batch is rejected at validation with a descriptive error.
    assert_eq!(
        result,
        Err(Ok(MilestoneError::MilestoneToApproveDoesNotExist))
    );

    // Atomicity: M0 and M1 must NOT have been approved despite being valid.
    let e = s.client.get_escrow();
    for i in 0..e.milestones.len() {
        let m = e.milestones.get(i).unwrap();
        assert_eq!(m.approvals.approval_count, 0, "M{i} must not be approved");
        assert!(
            m.approvals.approved_by.is_empty(),
            "M{i} approved_by must be empty"
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario B: Batch release [0, 1, 2] where M1 is unapproved.
// ---------------------------------------------------------------------------
#[test]
fn scenario_b_release_with_unapproved_milestone_is_atomic() {
    let env = Env::default();
    let s = build_escrow(&env, &[10_000_000, 10_000_000, 10_000_000], 1, 500);
    let total = 30_000_000;
    fund(&s, total);

    // Approve only M0 and M2; leave M1 unapproved.
    s.client
        .approve_milestones(&vec![&env, 0u32, 2u32], &s.approver);

    let r0 = receiver(&s, 0);
    let r1 = receiver(&s, 1);
    let r2 = receiver(&s, 2);

    let result = s.client.try_release_funds(
        &s.release_signer,
        &s.trustless_work,
        &vec![&env, 0u32, 1u32, 2u32],
    );

    // Entire batch fails because M1 is not approved.
    assert_eq!(result, Err(Ok(ReleaseError::EscrowNotCompleted)));

    // Atomicity: nothing released, no funds moved to ANY receiver.
    let e = s.client.get_escrow();
    for i in 0..e.milestones.len() {
        assert!(
            !e.milestones.get(i).unwrap().released,
            "M{i} must not be released"
        );
    }
    assert_eq!(s.token.balance(&r0), 0, "M0 receiver must not be paid");
    assert_eq!(s.token.balance(&r1), 0, "M1 receiver must not be paid");
    assert_eq!(s.token.balance(&r2), 0, "M2 receiver must not be paid");
    assert_eq!(
        s.token.balance(&s.client.address),
        total,
        "contract balance must be unchanged"
    );
    assert_eq!(s.token.balance(&s.trustless_work), 0);
    assert_eq!(s.token.balance(&s.platform), 0);
}

// ---------------------------------------------------------------------------
// Scenario C: Batch release [0, 1, 2] where M0 was already released.
// ---------------------------------------------------------------------------
#[test]
fn scenario_c_release_with_already_released_milestone_is_atomic() {
    let env = Env::default();
    let s = build_escrow(&env, &[10_000_000, 10_000_000, 10_000_000], 1, 500);
    let total = 30_000_000;
    fund(&s, total);

    // Approve all, then release M0 on its own.
    s.client
        .approve_milestones(&vec![&env, 0u32, 1u32, 2u32], &s.approver);
    s.client
        .release_funds(&s.release_signer, &s.trustless_work, &vec![&env, 0u32]);

    let r0 = receiver(&s, 0);
    let r1 = receiver(&s, 1);
    let r2 = receiver(&s, 2);

    let burned_after_individual = s.token.balance(&s.messenger);
    assert!(
        burned_after_individual > 0,
        "the messenger should hold M0's burned funds from the individual release"
    );
    assert_eq!(s.token.balance(&r0), 0);
    assert_eq!(s.token.balance(&r1), 0);
    assert_eq!(s.token.balance(&r2), 0);

    // Attempt to batch-release including the already-released M0.
    let result = s.client.try_release_funds(
        &s.release_signer,
        &s.trustless_work,
        &vec![&env, 0u32, 1u32, 2u32],
    );

    // The error on M0 aborts the whole batch.
    assert_eq!(result, Err(Ok(ReleaseError::MilestoneAlreadyReleased)));

    // Atomicity: M1 and M2 must remain unreleased and unpaid.
    let e = s.client.get_escrow();
    assert!(e.milestones.get(0).unwrap().released, "M0 stays released");
    assert!(
        !e.milestones.get(1).unwrap().released,
        "M1 must NOT be released"
    );
    assert!(
        !e.milestones.get(2).unwrap().released,
        "M2 must NOT be released"
    );
    assert_eq!(s.token.balance(&r1), 0, "M1 receiver must not be paid");
    assert_eq!(s.token.balance(&r2), 0, "M2 receiver must not be paid");
    assert_eq!(
        s.token.balance(&s.messenger),
        burned_after_individual,
        "burned total must be unchanged by the failed batch"
    );
    assert_eq!(
        s.token.balance(&s.client.address),
        total - 10_000_000,
        "only M0's amount left the contract"
    );
}

// ---------------------------------------------------------------------------
// Scenario D: Batch release all 5 milestones (distinct amounts) at once.
// ---------------------------------------------------------------------------
#[test]
fn scenario_d_release_all_milestones_distributes_correctly() {
    let env = Env::default();
    let amounts = [
        10_000_000i128,
        20_000_000,
        30_000_000,
        40_000_000,
        50_000_000,
    ];
    let platform_fee = 500u32; // 5%
    let s = build_escrow(&env, &amounts, 1, platform_fee);
    let total: i128 = amounts.iter().sum();
    fund(&s, total);

    s.client
        .approve_milestones(&vec![&env, 0u32, 1u32, 2u32, 3u32, 4u32], &s.approver);
    s.client.release_funds(
        &s.release_signer,
        &s.trustless_work,
        &vec![&env, 0u32, 1u32, 2u32, 3u32, 4u32],
    );

    // Fees are computed per milestone; amounts are multiples of 10_000 so the
    // integer division is exact (no rounding ambiguity).
    let mut tw_total = 0i128;
    let mut platform_total = 0i128;
    let mut net_total = 0i128;
    for (i, amount) in amounts.iter().enumerate() {
        let tw_fee = amount * TRUSTLESS_WORK_FEE_BPS / BASIS_POINTS_DENOMINATOR;
        let platform_cut = amount * platform_fee as i128 / BASIS_POINTS_DENOMINATOR;
        let net = amount - tw_fee - platform_cut;
        tw_total += tw_fee;
        platform_total += platform_cut;

        net_total += net;
        let r = receiver(&s, i as u32);
        assert_eq!(
            s.token.balance(&r),
            0,
            "M{i} receiver must not hold funds; the payout burns via CCTP"
        );
    }

    assert_eq!(
        s.token.balance(&s.messenger),
        net_total,
        "the messenger must hold every burned net amount"
    );

    assert_eq!(
        s.token.balance(&s.trustless_work),
        tw_total,
        "Trustless Work fee total mismatch"
    );
    assert_eq!(
        s.token.balance(&s.platform),
        platform_total,
        "platform fee total mismatch"
    );
    assert_eq!(
        s.token.balance(&s.client.address),
        0,
        "contract must be fully drained after releasing all milestones"
    );

    let e = s.client.get_escrow();
    for i in 0..e.milestones.len() {
        assert!(
            e.milestones.get(i).unwrap().released,
            "M{i} must be released"
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario E: Empty batches must return descriptive errors, not silent no-ops.
// ---------------------------------------------------------------------------
#[test]
fn scenario_e_empty_batches_return_descriptive_errors() {
    let env = Env::default();
    let s = build_escrow(&env, &[10_000_000, 10_000_000, 10_000_000], 1, 500);
    fund(&s, 30_000_000);

    let empty: Vec<u32> = vec![&env];

    let approve_res = s.client.try_approve_milestones(&empty, &s.approver);
    assert_eq!(
        approve_res,
        Err(Ok(MilestoneError::BatchMilestoneApproveEmpty))
    );

    let release_res = s
        .client
        .try_release_funds(&s.release_signer, &s.trustless_work, &empty);
    assert_eq!(release_res, Err(Ok(ReleaseError::ReleaseMilestonesEmpty)));

    let dispute_res =
        s.client
            .try_dispute_milestones(&s.approver, &empty, &String::from_str(&env, "reason"));
    assert_eq!(
        dispute_res,
        Err(Ok(EscrowError::BatchMilestoneDisputeEmpty))
    );

    // No silent mutation occurred.
    let e = s.client.get_escrow();
    for i in 0..e.milestones.len() {
        let m = e.milestones.get(i).unwrap();
        assert_eq!(m.approvals.approval_count, 0);
        assert!(!m.released);
        assert!(!m.dispute.is_disputed);
    }
}

// ---------------------------------------------------------------------------
// Scenario F: Batch dispute [0, 1] where M0 is already disputed.
// ---------------------------------------------------------------------------
#[test]
fn scenario_f_dispute_with_already_disputed_milestone_is_atomic() {
    let env = Env::default();
    let s = build_escrow(&env, &[10_000_000, 10_000_000, 10_000_000], 1, 500);
    fund(&s, 30_000_000);

    // Dispute M0 on its own. The approver is authorized to dispute and is not a
    // dispute resolver.
    s.client.dispute_milestones(
        &s.approver,
        &vec![&env, 0u32],
        &String::from_str(&env, "reason"),
    );

    // Attempt to batch-dispute [0, 1] where M0 is already in dispute.
    let result = s.client.try_dispute_milestones(
        &s.approver,
        &vec![&env, 0u32, 1u32],
        &String::from_str(&env, "reason"),
    );

    assert_eq!(result, Err(Ok(EscrowError::MilestoneAlreadyDisputed)));

    // Atomicity: M1 must NOT have been disputed despite being valid.
    let e = s.client.get_escrow();
    assert!(
        e.milestones.get(0).unwrap().dispute.is_disputed,
        "M0 stays disputed"
    );
    assert!(
        !e.milestones.get(1).unwrap().dispute.is_disputed,
        "M1 must NOT be disputed"
    );
    assert!(
        !e.milestones.get(2).unwrap().dispute.is_disputed,
        "M2 must remain untouched"
    );
}
