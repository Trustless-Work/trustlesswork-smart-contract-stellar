extern crate std;

use crate::modules::cctp::constants::{cctp_forward_hook_data, CCTP_TOKEN_MESSENGER_STRKEY};
use crate::modules::cctp::release::{
    release_receiver_amount_via_cctp_forwarding_with_messenger,
    release_receiver_amount_via_cctp_with_messenger,
};
use crate::storage::types::{
    Dispute, Escrow, Milestone, MilestoneApprovals, MilestoneUpdate, Roles, Trustline,
};
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, token, vec, Address, Bytes, BytesN, Env,
    String,
};
use token::Client as TokenClient;

use super::helpers::{create_escrow_contract, create_usdc_token};

/// Mock standing in for Circle's TokenMessengerMinter: requires the caller's
/// auth and pulls the approved USDC from the caller via the allowance.
#[contract]
pub struct MockTokenMessenger;

#[contractimpl]
impl MockTokenMessenger {
    #[allow(clippy::too_many_arguments)]
    pub fn deposit_for_burn(
        e: Env,
        caller: Address,
        amount: i128,
        _destination_domain: u32,
        _mint_recipient: BytesN<32>,
        burn_token: Address,
        _destination_caller: BytesN<32>,
        _max_fee: i128,
        _min_finality_threshold: u32,
    ) {
        caller.require_auth();
        let self_addr = e.current_contract_address();
        TokenClient::new(&e, &burn_token).transfer_from(&self_addr, &caller, &self_addr, &amount);
    }

    /// Same as `deposit_for_burn`, but also asserts the hook data matches
    /// the reserved `cctp-forward` magic bytes, so a regression that mangles
    /// the encoding fails the test instead of silently burning funds with
    /// a broken forward request.
    #[allow(clippy::too_many_arguments)]
    pub fn deposit_for_burn_with_hook(
        e: Env,
        caller: Address,
        amount: i128,
        _destination_domain: u32,
        _mint_recipient: BytesN<32>,
        burn_token: Address,
        _destination_caller: BytesN<32>,
        _max_fee: i128,
        _min_finality_threshold: u32,
        hook_data: Bytes,
    ) {
        caller.require_auth();
        assert_eq!(hook_data, cctp_forward_hook_data(&e));
        let self_addr = e.current_contract_address();
        TokenClient::new(&e, &burn_token).transfer_from(&self_addr, &caller, &self_addr, &amount);
    }

    /// Fixed protocol fee for tests — real value comes from Circle's live
    /// `TokenMessengerMinter` config, irrelevant to what this mock verifies.
    pub fn get_min_fee_amount(_e: Env, _burn_token: Address, _amount: i128) -> i128 {
        0
    }
}

fn evm_recipient(e: &Env, byte: u8) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[31] = byte;
    BytesN::from_array(e, &bytes)
}

fn milestone(env: &Env, amount: i128, receiver: &Address) -> Milestone {
    Milestone {
        description: String::from_str(env, "M"),
        status: String::from_str(env, "Completed"),
        evidence: String::from_str(env, "Done"),
        approvals: MilestoneApprovals {
            target: 1,
            approval_count: 0,
            approved_by: vec![env],
        },
        amount,
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(env, ""),
            resolved: false,
        },
        released: false,
        receiver: crate::tests::helpers::test_receiver(&env, &receiver),
    }
}

struct EscrowFixture {
    escrow: Escrow,
    admin: Address,
    approver: Address,
    release_signer: Address,
    platform: Address,
    receiver0: Address,
    receiver1: Address,
}

/// Two milestones with DIFFERENT receivers, to prove per-milestone routing.
fn base_escrow(env: &Env, usdc: &Address, amount_each: i128, platform_fee: u32) -> EscrowFixture {
    let admin = Address::generate(env);
    let approver = Address::generate(env);
    let release_signer = Address::generate(env);
    let platform = Address::generate(env);
    let receiver0 = Address::generate(env);
    let receiver1 = Address::generate(env);

    let roles = Roles {
        approvers: vec![env, approver.clone()],
        service_providers: vec![env, Address::generate(env)],
        platform: platform.clone(),
        release_signers: vec![env, release_signer.clone()],
        dispute_resolvers: vec![env, Address::generate(env)],
        admin: admin.clone(),
        observers: vec![env],
    };

    let escrow = Escrow {
        engagement_id: String::from_str(env, "cctp_multi"),
        title: String::from_str(env, "CCTP Multi"),
        description: String::from_str(env, "Per-milestone cross-chain"),
        roles,
        platform_fee,
        milestones: vec![
            env,
            milestone(env, amount_each, &receiver0),
            milestone(env, amount_each, &receiver1),
        ],
        trustline: Trustline {
            address: usdc.clone(),
        },
        receiver_memo: 0,
    };

    EscrowFixture {
        escrow,
        admin,
        approver,
        release_signer,
        platform,
        receiver0,
        receiver1,
    }
}

fn net_of(amount: i128, platform_fee: u32) -> (i128, i128, i128) {
    let tw = amount * 30 / 10_000;
    let plat = amount * platform_fee as i128 / 10_000;
    (tw, plat, amount - tw - plat)
}

#[test]
fn release_burns_via_destination_registered_at_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let messenger = Address::from_str(&env, CCTP_TOKEN_MESSENGER_STRKEY);
    env.register_at(&messenger, MockTokenMessenger, ());

    let each: i128 = 50_000_000;
    let platform_fee: u32 = 500;
    let f = base_escrow(&env, &usdc.0.address, each, platform_fee);
    let tw_address = Address::generate(&env);

    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);
    usdc.1.mint(&client.address, &(each * 2));

    // No set_cross_chain_destination call: the destination provided at
    // initialize_escrow is enough for the release to burn via CCTP.
    client.approve_milestones(&vec![&env, 0u32], &f.approver);
    client.release_funds(
        &f.release_signer,
        &tw_address,
        &vec![&env, 0u32],
        &vec![&env, 0i128],
    );

    let (_tw, _plat, net) = net_of(each, platform_fee);
    assert_eq!(usdc.0.balance(&messenger), net);
    assert_eq!(usdc.0.balance(&f.receiver0), 0);
}

#[test]
fn initialize_rejects_invalid_destination_domain_and_zero_recipient() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let mut f = base_escrow(&env, &usdc.0.address, 50_000_000, 500);
    let mut m0 = f.escrow.milestones.get(0).unwrap();
    m0.receiver.destination_domain = 999;
    f.escrow.milestones.set(0, m0);
    let client = create_escrow_contract(&env, &f.admin).client;
    let res = client.try_initialize_escrow(&f.escrow);
    assert!(res.is_err(), "invalid destination domain must fail at init");

    let mut f = base_escrow(&env, &usdc.0.address, 50_000_000, 500);
    let mut m1 = f.escrow.milestones.get(1).unwrap();
    m1.receiver.mint_recipient = BytesN::from_array(&env, &[0u8; 32]);
    f.escrow.milestones.set(1, m1);
    let client = create_escrow_contract(&env, &f.admin).client;
    let res = client.try_initialize_escrow(&f.escrow);
    assert!(res.is_err(), "zero mint recipient must fail at init");
}

#[test]
fn admin_updates_destination_via_manage_milestones_without_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let each: i128 = 50_000_000;
    let f = base_escrow(&env, &usdc.0.address, each, 500);

    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);

    // Without funds the admin can retarget a milestone's destination.
    let updates = vec![
        &env,
        MilestoneUpdate {
            index: 0u32,
            new_description: None,
            new_amount: None,
            new_destination_domain: Some(6),
            new_mint_recipient: Some(evm_recipient(&env, 0xAB)),
        },
    ];
    client.manage_milestones(&f.admin, &vec![&env], &updates);
    let stored = client.get_escrow().milestones.get(0).unwrap().receiver;
    assert_eq!(stored.destination_domain, 6);
    assert_eq!(stored.mint_recipient, evm_recipient(&env, 0xAB));

    // Half a destination is rejected.
    let updates = vec![
        &env,
        MilestoneUpdate {
            index: 0u32,
            new_description: None,
            new_amount: None,
            new_destination_domain: Some(1),
            new_mint_recipient: None,
        },
    ];
    let res = client.try_manage_milestones(&f.admin, &vec![&env], &updates);
    assert!(res.is_err(), "half a destination must be rejected");

    // With funds milestone updates are rejected entirely, destination included.
    let funder = Address::generate(&env);
    let funded_escrow = client.get_escrow();
    usdc.1.mint(&funder, &(each * 2));
    client.fund_escrow(&funder, &funded_escrow, &(each * 2));
    let updates = vec![
        &env,
        MilestoneUpdate {
            index: 0u32,
            new_description: None,
            new_amount: None,
            new_destination_domain: Some(7),
            new_mint_recipient: Some(evm_recipient(&env, 0xCD)),
        },
    ];
    let res = client.try_manage_milestones(&f.admin, &vec![&env], &updates);
    assert!(res.is_err(), "destination must be frozen once funded");
}

#[test]
fn release_rejects_max_fee_exceeding_cap() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let messenger = Address::from_str(&env, CCTP_TOKEN_MESSENGER_STRKEY);
    env.register_at(&messenger, MockTokenMessenger, ());

    // milestone amount = 50_000_000 -> cap is amount / 10 = 5_000_000.
    let each: i128 = 50_000_000;
    let platform_fee: u32 = 500;
    let f = base_escrow(&env, &usdc.0.address, each, platform_fee);
    let tw_address = Address::generate(&env);

    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);
    usdc.1.mint(&client.address, &(each * 2));
    client.approve_milestones(&vec![&env, 0u32], &f.approver);

    // Above the cap, negative, and a fee vec whose length mismatches the
    // indices are all rejected.
    let too_high = client.try_release_funds(
        &f.release_signer,
        &tw_address,
        &vec![&env, 0u32],
        &vec![&env, each / 10 + 1],
    );
    assert!(too_high.is_err(), "max_fee above the cap must fail");
    let negative = client.try_release_funds(
        &f.release_signer,
        &tw_address,
        &vec![&env, 0u32],
        &vec![&env, -1i128],
    );
    assert!(negative.is_err(), "negative max_fee must fail");
    let mismatched = client.try_release_funds(
        &f.release_signer,
        &tw_address,
        &vec![&env, 0u32],
        &vec![&env, 0i128, 0i128],
    );
    assert!(mismatched.is_err(), "max_fees length mismatch must fail");

    // Exactly at the cap is fine.
    client.release_funds(
        &f.release_signer,
        &tw_address,
        &vec![&env, 0u32],
        &vec![&env, each / 10],
    );
    let (_tw, _plat, net) = net_of(each, platform_fee);
    assert_eq!(usdc.0.balance(&messenger), net);
}

#[test]
fn helper_sends_seventh_decimal_remainder_to_stellar() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let escrow_contract = env.register(
        crate::contract::EscrowContract,
        (&admin, &BytesN::from_array(&env, &[0u8; 32])),
    );
    let amount: i128 = 1_0000003;
    usdc.1.mint(&escrow_contract, &amount);

    let mock_messenger = env.register(MockTokenMessenger, ());
    let stellar_receiver = Address::generate(&env);

    release_receiver_amount_via_cctp_with_messenger(
        &env,
        &usdc.0,
        &escrow_contract,
        &mock_messenger,
        &usdc.0.address,
        amount,
        6,
        &evm_recipient(&env, 0xAB),
        &stellar_receiver,
    );

    assert_eq!(usdc.0.balance(&mock_messenger), 1_0000000);
    assert_eq!(usdc.0.balance(&stellar_receiver), 3);
    assert_eq!(usdc.0.balance(&escrow_contract), 0);
}

#[test]
fn forwarding_helper_sends_seventh_decimal_remainder_to_stellar() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let escrow_contract = env.register(
        crate::contract::EscrowContract,
        (&admin, &BytesN::from_array(&env, &[0u8; 32])),
    );
    let amount: i128 = 1_0000003;
    usdc.1.mint(&escrow_contract, &amount);

    let mock_messenger = env.register(MockTokenMessenger, ());
    let stellar_receiver = Address::generate(&env);

    release_receiver_amount_via_cctp_forwarding_with_messenger(
        &env,
        &usdc.0,
        &escrow_contract,
        &mock_messenger,
        &usdc.0.address,
        amount,
        6,
        &evm_recipient(&env, 0xAB),
        1000,
        &stellar_receiver,
    );

    // Same burn/remainder split as the plain path — forwarding only changes
    // `max_fee`/`hook_data`, not how much gets burned vs. sent directly.
    assert_eq!(usdc.0.balance(&mock_messenger), 1_0000000);
    assert_eq!(usdc.0.balance(&stellar_receiver), 3);
    assert_eq!(usdc.0.balance(&escrow_contract), 0);
}
