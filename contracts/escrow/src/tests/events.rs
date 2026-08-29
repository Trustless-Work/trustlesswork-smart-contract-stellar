extern crate std;

// Audit-trail coverage for the events that carry *what changed*, not just
// that something changed. Each test drives an entrypoint and then asserts on
// the concrete event payload emitted, proving an events-only indexer could
// reconstruct the change without diffing a full storage snapshot.

use crate::events::handler::{EscrowUpdated, MilestoneStatusChanged, MilestonesManaged};
use crate::storage::types::{
    Dispute, Escrow, EscrowPropertyChanges, Milestone, MilestoneAddedEntry, MilestoneApprovals,
    MilestoneStatusEntry, MilestoneStatusUpdate, MilestoneUpdate, MilestoneUpdatedEntry, Roles,
    Trustline,
};
use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{vec, xdr, Address, Bytes, BytesN, Env, Event, String, TryFromVal};

use super::helpers::{create_escrow_contract, create_usdc_token, TestData};

/// SHA-256 computed independently of the contract, so tests verify the
/// emitted hash rather than echo it.
fn sha256(env: &Env, data: &[u8]) -> BytesN<32> {
    env.crypto()
        .sha256(&Bytes::from_slice(env, data))
        .to_bytes()
}

/// XDR payload of the last emitted event (always the escrow event here).
fn last_event_data(env: &Env) -> xdr::ScVal {
    let all = env.events().all();
    let raw = all.events();
    let last = raw.last().expect("expected at least one emitted event");
    match &last.body {
        xdr::ContractEventBody::V0(v0) => v0.data.clone(),
    }
}

fn last_event_topics(env: &Env) -> std::vec::Vec<xdr::ScVal> {
    let all = env.events().all();
    let raw = all.events();
    let last = raw.last().expect("expected at least one emitted event");
    match &last.body {
        xdr::ContractEventBody::V0(v0) => v0.topics.to_vec(),
    }
}

/// Asserts the last event matches `expected` in both topics and data.
fn assert_last_event<E: Event>(env: &Env, expected: &E) {
    let expected_data =
        xdr::ScVal::try_from_val(env, &expected.data(env)).expect("event data -> ScVal");
    assert_eq!(
        last_event_data(env),
        expected_data,
        "event data payload mismatch"
    );

    let expected_topics: std::vec::Vec<xdr::ScVal> = expected
        .topics(env)
        .iter()
        .map(|t| xdr::ScVal::try_from_val(env, &t).expect("topic -> ScVal"))
        .collect();
    assert_eq!(
        last_event_topics(env),
        expected_topics,
        "event topics mismatch"
    );
}

struct Fixture<'a> {
    env: Env,
    client: TestData<'a>,
    escrow_admin: Address,
    service_provider: Address,
    receiver: Address,
    engagement_id: String,
}

/// Initializes an escrow with one in-progress milestone for event tests.
fn setup(engagement: &str) -> Fixture<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let approver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let escrow_admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);

    let (token_client, _) = create_usdc_token(&env, &token_admin);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![&env],
            },
            amount: 100_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(&env, ""),
                resolved: false,
            },
            released: false,
            receiver: receiver.clone(),
        },
    ];

    let roles = Roles {
        approvers: vec![&env, approver.clone()],
        service_providers: vec![&env, service_provider.clone()],
        platform: platform.clone(),
        release_signers: vec![&env, release_signer.clone()],
        dispute_resolvers: vec![&env, dispute_resolver.clone()],
        admin: escrow_admin.clone(),
        observers: vec![&env],
    };

    let engagement_id = String::from_str(&env, engagement);
    let props = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Original Title"),
        description: String::from_str(&env, "Original Description"),
        roles: roles.clone(),
        platform_fee: 300,
        milestones: milestones.clone(),
        trustline: Trustline {
            address: token_client.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env, &escrow_admin);
    client.client.initialize_escrow(&props);

    Fixture {
        env,
        client,
        escrow_admin,
        service_provider,
        receiver,
        engagement_id,
    }
}

#[test]
fn milestone_status_changed_event_carries_evidence_hash() {
    let f = setup("evt_status");
    let env = &f.env;

    let evidence = "IPFS://proof-of-completion";
    let updates = vec![
        env,
        MilestoneStatusUpdate {
            milestone_index: 0,
            new_status: String::from_str(env, "completed"),
            new_evidence: Some(String::from_str(env, evidence)),
        },
    ];
    f.client
        .client
        .change_milestone_status(&updates, &f.service_provider);

    let expected = MilestoneStatusChanged {
        engagement_id: f.engagement_id.clone(),
        service_provider: f.service_provider.clone(),
        updates: vec![
            env,
            MilestoneStatusEntry {
                index: 0,
                status: String::from_str(env, "completed"),
                evidence_hash: Some(sha256(env, evidence.as_bytes())),
            },
        ],
    };
    assert_last_event(env, &expected);
}

#[test]
fn milestone_status_changed_event_omits_hash_when_evidence_unchanged() {
    let f = setup("evt_status_noev");
    let env = &f.env;

    // No new evidence supplied -> the entry must carry `None`, distinguishing a
    // status-only change from one that also recorded fresh evidence.
    let updates = vec![
        env,
        MilestoneStatusUpdate {
            milestone_index: 0,
            new_status: String::from_str(env, "under-review"),
            new_evidence: None,
        },
    ];
    f.client
        .client
        .change_milestone_status(&updates, &f.service_provider);

    let expected = MilestoneStatusChanged {
        engagement_id: f.engagement_id.clone(),
        service_provider: f.service_provider.clone(),
        updates: vec![
            env,
            MilestoneStatusEntry {
                index: 0,
                status: String::from_str(env, "under-review"),
                evidence_hash: None,
            },
        ],
    };
    assert_last_event(env, &expected);
}

#[test]
fn oversized_evidence_still_errors_gracefully_before_hashing() {
    // The event carries a hash of the evidence, but hashing must not run until
    // the update has been validated — otherwise an over-long evidence string
    // would trap in the hasher instead of returning `StringTooLong`.
    let f = setup("evt_status_oversized");
    let env = &f.env;

    let too_long = String::from_str(env, std::str::from_utf8(&[b'a'; 501]).unwrap());
    let updates = vec![
        env,
        MilestoneStatusUpdate {
            milestone_index: 0,
            new_status: String::from_str(env, "completed"),
            new_evidence: Some(too_long),
        },
    ];
    let result = f
        .client
        .client
        .try_change_milestone_status(&updates, &f.service_provider);
    assert_eq!(
        result.err(),
        Some(Ok(crate::error::MilestoneError::StringTooLong))
    );
}

#[test]
fn escrow_updated_event_reports_changed_properties() {
    let f = setup("evt_update");
    let env = &f.env;

    let current = f.client.client.get_escrow();

    // Change the title and platform fee before funding; leave everything else
    // untouched so the change flags must be exactly {title, platform_fee}.
    let mut new_props = current.clone();
    new_props.title = String::from_str(env, "Renamed Escrow");
    new_props.platform_fee = 500;

    f.client.client.update_escrow(&f.escrow_admin, &new_props);

    let expected = EscrowUpdated {
        engagement_id: f.engagement_id.clone(),
        admin: f.escrow_admin.clone(),
        changes: EscrowPropertyChanges {
            engagement_id: false,
            title: true,
            description: false,
            platform_fee: true,
            roles: false,
            trustline: false,
            receiver_memo: false,
            old_platform_fee: 300,
            new_platform_fee: 500,
        },
    };
    assert_last_event(env, &expected);
}

#[test]
fn manage_milestones_event_details_added_and_updated() {
    let f = setup("evt_manage");
    let env = &f.env;

    // Update milestone 0's description + amount, and append one new milestone.
    let updates = vec![
        env,
        MilestoneUpdate {
            index: 0,
            new_description: Some(String::from_str(env, "Reworded milestone")),
            new_amount: Some(75_000_000),
        },
    ];
    let added = vec![
        env,
        Milestone {
            description: String::from_str(env, "Freshly added milestone"),
            status: String::from_str(env, "Pending"),
            evidence: String::from_str(env, ""),
            approvals: MilestoneApprovals {
                target: 1,
                approval_count: 0,
                approved_by: vec![env],
            },
            amount: 25_000_000,
            dispute: Dispute {
                is_disputed: false,
                reason: String::from_str(env, ""),
                resolved: false,
            },
            released: false,
            receiver: f.receiver.clone(),
        },
    ];

    f.client
        .client
        .manage_milestones(&f.escrow_admin, &added, &updates);

    let expected = MilestonesManaged {
        engagement_id: f.engagement_id.clone(),
        admin: f.escrow_admin.clone(),
        added_count: 1,
        updated_count: 1,
        added: vec![
            env,
            MilestoneAddedEntry {
                // Appended after the single pre-existing milestone (index 0).
                index: 1,
                amount: 25_000_000,
                description_hash: sha256(env, b"Freshly added milestone"),
            },
        ],
        updated: vec![
            env,
            MilestoneUpdatedEntry {
                index: 0,
                new_amount: Some(75_000_000),
                new_description_hash: Some(sha256(env, b"Reworded milestone")),
            },
        ],
    };
    assert_last_event(env, &expected);
}
