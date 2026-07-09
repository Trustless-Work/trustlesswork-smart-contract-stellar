extern crate std;

use crate::error::CctpError;
use crate::modules::cctp::constants::{cctp_forward_hook_data, CCTP_TOKEN_MESSENGER_STRKEY};
use crate::modules::cctp::release::{
    release_receiver_amount_via_cctp_forwarding_with_messenger,
    release_receiver_amount_via_cctp_with_messenger,
};
use crate::storage::types::{
    Dispute, Escrow, Milestone, MilestoneApprovals, Roles, Trustline,
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

struct EscrowFixture {
    escrow: Escrow,
    receiver: Address,
    admin: Address,
    approver: Address,
    release_signer: Address,
    platform: Address,
}

fn base_escrow(env: &Env, usdc: &Address, amount: i128, platform_fee: u32) -> EscrowFixture {
    let admin = Address::generate(env);
    let approver = Address::generate(env);
    let release_signer = Address::generate(env);
    let platform = Address::generate(env);
    let receiver = Address::generate(env);

    let roles = Roles {
        approvers: vec![env, approver.clone()],
        service_providers: vec![env, Address::generate(env)],
        platform: platform.clone(),
        release_signers: vec![env, release_signer.clone()],
        dispute_resolvers: vec![env, Address::generate(env)],
        receiver: receiver.clone(),
        admin: admin.clone(),
        observers: vec![env],
    };

    let escrow = Escrow {
        engagement_id: String::from_str(env, "cctp_test"),
        title: String::from_str(env, "CCTP Test"),
        description: String::from_str(env, "Cross-chain release"),
        roles,
        amount,
        platform_fee,
        milestones: vec![
            env,
            Milestone {
                description: String::from_str(env, "M1"),
                status: String::from_str(env, "Completed"),
                evidence: String::from_str(env, "Done"),
                approvals: MilestoneApprovals {
                    target: 1,
                    approval_count: 0,
                    approved_by: vec![env],
                },
            },
        ],
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(env, ""),
            resolved: false,
        },
        released: false,
        trustline: Trustline {
            address: usdc.clone(),
        },
        receiver_memo: 0,
    };

    EscrowFixture {
        escrow,
        receiver,
        admin,
        approver,
        release_signer,
        platform,
    }
}

#[test]
fn release_routes_to_cctp_when_receiver_registered_destination() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let messenger = Address::from_str(&env, CCTP_TOKEN_MESSENGER_STRKEY);
    env.register_at(&messenger, MockTokenMessenger, ());

    let amount: i128 = 100_000_000;
    let platform_fee: u32 = 500;
    let f = base_escrow(&env, &usdc.0.address, amount, platform_fee);
    let tw_address = Address::generate(&env);

    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);
    usdc.1.mint(&client.address, &amount);

    client.set_cross_chain_destination(&f.receiver, &6, &evm_recipient(&env, 0xAB));

    client.approve_milestones(&vec![&env, 0u32], &f.approver);
    client.release_funds(&f.release_signer, &tw_address);

    let tw_fee = amount * 30 / 10_000;
    let platform_commission = amount * platform_fee as i128 / 10_000;
    let receiver_amount = amount - tw_fee - platform_commission;

    assert_eq!(usdc.0.balance(&tw_address), tw_fee);
    assert_eq!(usdc.0.balance(&f.platform), platform_commission);
    assert_eq!(usdc.0.balance(&messenger), receiver_amount);
    assert_eq!(usdc.0.balance(&f.receiver), 0);
    assert_eq!(usdc.0.balance(&client.address), 0);
}

#[test]
fn release_stays_on_stellar_without_registered_destination() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    let platform_fee: u32 = 500;
    let f = base_escrow(&env, &usdc.0.address, amount, platform_fee);
    let tw_address = Address::generate(&env);

    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);
    usdc.1.mint(&client.address, &amount);

    client.approve_milestones(&vec![&env, 0u32], &f.approver);
    client.release_funds(&f.release_signer, &tw_address);

    let receiver_amount =
        amount - (amount * 30 / 10_000) - (amount * platform_fee as i128 / 10_000);
    assert_eq!(usdc.0.balance(&f.receiver), receiver_amount);
}

#[test]
fn only_receiver_can_set_destination() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    let f = base_escrow(&env, &usdc.0.address, amount, 500);

    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);

    let res = client.try_set_cross_chain_destination(
        &f.release_signer,
        &6,
        &evm_recipient(&env, 0xAB),
    );
    assert_eq!(
        res,
        Err(Ok(CctpError::OnlyReceiverCanSetDestination))
    );
}

#[test]
fn set_destination_rejects_invalid_domain() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    let f = base_escrow(&env, &usdc.0.address, amount, 500);

    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);

    let res =
        client.try_set_cross_chain_destination(&f.receiver, &999, &evm_recipient(&env, 0xAB));
    assert_eq!(res, Err(Ok(CctpError::InvalidDestinationDomain)));
}

#[test]
fn set_destination_rejects_zero_recipient() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    let f = base_escrow(&env, &usdc.0.address, amount, 500);

    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);

    let res = client.try_set_cross_chain_destination(
        &f.receiver,
        &6,
        &BytesN::from_array(&env, &[0u8; 32]),
    );
    assert_eq!(res, Err(Ok(CctpError::InvalidRecipient)));
}

#[test]
fn receiver_can_clear_destination_to_revert_to_stellar() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    let platform_fee: u32 = 500;
    let f = base_escrow(&env, &usdc.0.address, amount, platform_fee);
    let tw_address = Address::generate(&env);

    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);
    usdc.1.mint(&client.address, &amount);

    client.set_cross_chain_destination(&f.receiver, &6, &evm_recipient(&env, 0xAB));
    client.clear_cross_chain_destination(&f.receiver);

    client.approve_milestones(&vec![&env, 0u32], &f.approver);
    client.release_funds(&f.release_signer, &tw_address);

    let receiver_amount =
        amount - (amount * 30 / 10_000) - (amount * platform_fee as i128 / 10_000);
    assert_eq!(usdc.0.balance(&f.receiver), receiver_amount);
}

#[test]
fn helper_sends_seventh_decimal_remainder_to_stellar() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let escrow_contract = env.register(crate::contract::EscrowContract, (&admin, &BytesN::from_array(&env, &[0u8; 32])));
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
        &stellar_receiver,
    );

    // Same burn/remainder split as the plain path — forwarding only changes
    // `max_fee`/`hook_data`, not how much gets burned vs. sent directly.
    assert_eq!(usdc.0.balance(&mock_messenger), 1_0000000);
    assert_eq!(usdc.0.balance(&stellar_receiver), 3);
    assert_eq!(usdc.0.balance(&escrow_contract), 0);
}
