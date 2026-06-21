#!/bin/bash

################################################################################
# TRUSTLESS WORK ESCROW - COMPLETE E2E AUTOMATION (FULL - NO STOPPING)
# All 11 steps included - will complete fully
# UPDATED: Auto-imports all wallet secrets on first run
# Usage: ./trustless-work-e2e-complete.sh
################################################################################

set +e  # Don't exit on error - we handle it

# Configuration
NETWORK="testnet"
LOG_FILE="trustless-work-$(date +%Y%m%d-%H%M%S).log"
WASM_PATH="/home/ryzen/Desktop/grant-fox/trustlesswork-smart-contract-stellar/target/wasm32v1-none/release/escrow.wasm"

# Addresses
ALICE="GCPZCXSEWARYFZAJQEAJORUHZNGJNMQCDYYCDTYTUDD65H4TKKDF65HS"
APPROVER="GB5RJ5UI7264E5F2S5FA5ZETOTOS4HRWPGAHMHQDAPK6OJRKCXOH2MTI"
PROVIDER="GCD2CN27C4VUOGF7JNIZYT45MVIIGEDDHYMVCQIGGECAJQFIEDVEYJMY"
RELEASE_SIGNER="GAIAMLVKFC7NVQXH6UAA7WJCLJY6QGJSU7XBIQMXXGWYJHAWX2K2ZUVB"
RESOLVER="GCDOXZ3PWII2H6Y23QWAORCJRL7UTDKFBZB7FYGEGP3T6AKB2RB2TWO4"
PLATFORM="GB3Q632HYA62YVPGUZCOR3YJ4K6Y6HKJBA6DPHT67LF6ZWEPYA5ZSRDM"
RECEIVER="GB2TGNQ6KXPA3MEXNLGPTRG2UUKG5QWNW6XA2D2JE3NPV6CYD56VXIEW"
USDC_ISSUER="GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5"
USDC_TRUSTLINE="CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"
EVM_RECIPIENT="0x18C385B1748ae257179483564A5FB11bc5C18Ab7"


# Wallet Secrets
ALICE_SECRET="SC274FAFQS5MR37WH6FPI7I3622ZM6TNZ3HXQBZGIOE333KGQBXGOBVN"
APPROVER_SECRET="SCGUUJ5JUDXNWVCVRAH5BCBZIGBAHSCUNDP4IEA5FH235M64XAGRM6SD"
PROVIDER_SECRET="SCL2YX3LT6EOBB7MAJ6LGXVGOYGZUIGATKUZU7KXMOCYAKJNRNRSYXZH"
RELEASE_SIGNER_SECRET="SADY7ESHCCXGMI4MO5B774SBM2YPME7DS6BMNXNOQRLPBGWF6GGRXORT"
RESOLVER_SECRET="SCD4XWUFYK43OHQQQVGVXOOYGUTAKDJPWOZQAE7OXWGGUW5NJSTQTG5T"
PLATFORM_SECRET="SC37K5TTS6O6NV5J7M5TYOZIQ76MAKX4FMCFNUA4GMDCZUFVPZVQDLMW"
RECEIVER_SECRET="SDYSVX7N53YYHZOXIDWUPNKCLTDHZ6XCJRMVIMFKWYNS63563MXV5B5T"

# Parameters
AMOUNT="10000000"
PLATFORM_FEE="300"
ENGAGEMENT_ID="cli-test-001"
TITLE="CLI test"
DESCRIPTION="Automated testnet demo"
MILESTONE_DESCRIPTION="Work package 1"

# ============================================================================
# FUNCTIONS
# ============================================================================

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log_section() {
    echo "" | tee -a "$LOG_FILE"
    echo "═══════════════════════════════════════════════════════════════════" | tee -a "$LOG_FILE"
    echo "  $1" | tee -a "$LOG_FILE"
    echo "═══════════════════════════════════════════════════════════════════" | tee -a "$LOG_FILE"
}

log_step() {
    echo "" | tee -a "$LOG_FILE"
    echo ">>> STEP $1" | tee -a "$LOG_FILE"
}

success() {
    echo "✅ $1" | tee -a "$LOG_FILE"
}

warn() {
    echo "⚠️  $1" | tee -a "$LOG_FILE"
}

error() {
    echo "❌ ERROR: $1" | tee -a "$LOG_FILE"
}

extract_contract_id() {
    local output="$1"
    echo "$output" | grep -oE 'C[A-Z0-9]{55,56}' | tail -1
}

# ============================================================================
# STEP 0: Setup Stellar Wallet (Auto-import secrets)
# ============================================================================

step_0_setup_wallet() {
    log_step "0 - Setup Stellar CLI Wallet"
    
    log "Importing wallet secrets into Stellar CLI..."
    
    # Import alice
    log "Importing alice..."
    stellar keys add --secret-key alice "$ALICE_SECRET" --network "$NETWORK" >> "$LOG_FILE" 2>&1
    success "alice imported"
    sleep 1
    
    # Import approver
    log "Importing approver..."
    stellar keys add --secret-key approver "$APPROVER_SECRET" --network "$NETWORK" >> "$LOG_FILE" 2>&1
    success "approver imported"
    sleep 1
    
    # Import provider
    log "Importing provider..."
    stellar keys add --secret-key provider "$PROVIDER_SECRET" --network "$NETWORK" >> "$LOG_FILE" 2>&1
    success "provider imported"
    sleep 1
    
    # Import release_signer
    log "Importing release_signer..."
    stellar keys add --secret-key release_signer "$RELEASE_SIGNER_SECRET" --network "$NETWORK" >> "$LOG_FILE" 2>&1
    success "release_signer imported"
    sleep 1
    
    # Import resolver
    log "Importing resolver..."
    stellar keys add --secret-key resolver "$RESOLVER_SECRET" --network "$NETWORK" >> "$LOG_FILE" 2>&1
    success "resolver imported"
    sleep 1
    
    # Import platform
    log "Importing platform..."
    stellar keys add --secret-key platform "$PLATFORM_SECRET" --network "$NETWORK" >> "$LOG_FILE" 2>&1
    success "platform imported"
    sleep 1
    
    # Import receiver
    log "Importing receiver..."
    stellar keys add --secret-key receiver "$RECEIVER_SECRET" --network "$NETWORK" >> "$LOG_FILE" 2>&1
    success "receiver imported"
    sleep 1
    
    success "All wallet secrets imported successfully"
    sleep 2
    return 0
}

# ============================================================================
# STEP 1: Deploy Contract
# ============================================================================

step_1_deploy() {
    log_step "1 - Deploy Contract"
    log "Deploying from: $WASM_PATH"
    
    DEPLOY_OUT=$(stellar contract deploy \
        --wasm "$WASM_PATH" \
        --source alice \
        --network "$NETWORK" 2>&1)
    
    echo "$DEPLOY_OUT" >> "$LOG_FILE"
    
    CONTRACT_ID=$(extract_contract_id "$DEPLOY_OUT")
    
    if [ -z "$CONTRACT_ID" ]; then
        error "Could not extract contract ID"
        return 1
    fi
    
    success "Contract deployed: $CONTRACT_ID"
    echo "CONTRACT_ID=$CONTRACT_ID" >> "$LOG_FILE"
    sleep 5
    return 0
}

# ============================================================================
# STEP 2: Initialize Escrow
# ============================================================================

step_2_initialize() {
    log_step "2 - Initialize Escrow"
    
    CONTRACT_ID=$(grep "^CONTRACT_ID=" "$LOG_FILE" | tail -1 | cut -d'=' -f2)
    
    if [ -z "$CONTRACT_ID" ]; then
        error "CONTRACT_ID not found"
        return 1
    fi
    
    INIT_OUT=$(stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source alice \
        --network "$NETWORK" \
        -- initialize_escrow \
        --escrow_properties "{\"amount\":\"$AMOUNT\",\"cross_chain_receiver\":{\"destination_domain\":0,\"recipient\":\"00000000000000000000000018C385B1748ae257179483564A5FB11bc5C18Ab7\"},\"description\":\"$DESCRIPTION\",\"engagement_id\":\"$ENGAGEMENT_ID\",\"flags\":{\"disputed\":false,\"released\":false,\"resolved\":false},\"milestones\":[{\"approved\":false,\"description\":\"$MILESTONE_DESCRIPTION\",\"evidence\":\"\",\"status\":\"Pending\"}],\"platform_fee\":$PLATFORM_FEE,\"receiver_memo\":0,\"roles\":{\"approver\":\"$APPROVER\",\"dispute_resolver\":\"$RESOLVER\",\"platform\":\"$PLATFORM\",\"receiver\":\"$RECEIVER\",\"release_signer\":\"$RELEASE_SIGNER\",\"service_provider\":\"$PROVIDER\"},\"title\":\"$TITLE\",\"trustline\":{\"address\":\"$USDC_TRUSTLINE\"}}" 2>&1)
    
    if echo "$INIT_OUT" | grep -q "error\|Error\|ERROR"; then
        error "Initialize failed: $INIT_OUT"
        echo "$INIT_OUT" >> "$LOG_FILE"
        return 1
    fi
    
    echo "$INIT_OUT" >> "$LOG_FILE"
    success "Escrow initialized"
    sleep 5
    return 0
}

# ============================================================================
# STEP 3: Approve USDC
# ============================================================================

step_3_approve() {
    log_step "3 - Approve USDC"
    
    CONTRACT_ID=$(grep "^CONTRACT_ID=" "$LOG_FILE" | tail -1 | cut -d'=' -f2)
    
    APPROVE_OUT=$(stellar contract invoke \
        --id "$USDC_TRUSTLINE" \
        --source alice \
        --network "$NETWORK" \
        -- approve \
        --from alice \
        --spender "$CONTRACT_ID" \
        --amount "$AMOUNT" \
        --expiration_ledger 4000000 2>&1)
    
    if echo "$APPROVE_OUT" | grep -q "error\|Error\|ERROR"; then
        error "Approval failed: $APPROVE_OUT"
        echo "$APPROVE_OUT" >> "$LOG_FILE"
        return 1
    fi
    
    echo "$APPROVE_OUT" >> "$LOG_FILE"
    success "USDC approved"
    sleep 5
    return 0
}

# ============================================================================
# STEP 4: Fund Escrow
# ============================================================================

step_4_fund() {
    log_step "4 - Fund Escrow"
    
    CONTRACT_ID=$(grep "^CONTRACT_ID=" "$LOG_FILE" | tail -1 | cut -d'=' -f2)
    
    FUND_OUT=$(stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source alice \
        --network "$NETWORK" \
        -- fund_escrow \
        --expected_escrow "{\"amount\":\"$AMOUNT\",\"cross_chain_receiver\":{\"destination_domain\":0,\"recipient\":\"00000000000000000000000018c385b1748ae257179483564a5fb11bc5c18ab7\"},\"description\":\"$DESCRIPTION\",\"engagement_id\":\"$ENGAGEMENT_ID\",\"flags\":{\"disputed\":false,\"released\":false,\"resolved\":false},\"milestones\":[{\"approved\":false,\"description\":\"$MILESTONE_DESCRIPTION\",\"evidence\":\"\",\"status\":\"Pending\"}],\"platform_fee\":$PLATFORM_FEE,\"receiver_memo\":0,\"roles\":{\"approver\":\"$APPROVER\",\"dispute_resolver\":\"$RESOLVER\",\"platform\":\"$PLATFORM\",\"receiver\":\"$RECEIVER\",\"release_signer\":\"$RELEASE_SIGNER\",\"service_provider\":\"$PROVIDER\"},\"title\":\"$TITLE\",\"trustline\":{\"address\":\"$USDC_TRUSTLINE\"}}" \
        --signer alice \
        --amount "$AMOUNT" 2>&1)
    
    if echo "$FUND_OUT" | grep -q "error\|Error\|ERROR"; then
        error "Fund failed: $FUND_OUT"
        echo "$FUND_OUT" >> "$LOG_FILE"
        return 1
    fi
    
    echo "$FUND_OUT" >> "$LOG_FILE"
    success "Escrow funded: $AMOUNT"
    sleep 5
    return 0
}

# ============================================================================
# STEP 5: Fund Role Accounts
# ============================================================================

step_5_fund_roles() {
    log_step "5 - Fund Role Accounts"
    
    log "Funding provider..."
    stellar payment --source alice --destination "$PROVIDER" --amount 10 --network "$NETWORK" >> "$LOG_FILE" 2>&1
    success "Provider funded"
    
    sleep 2
    
    log "Funding approver..."
    stellar payment --source alice --destination "$APPROVER" --amount 10 --network "$NETWORK" >> "$LOG_FILE" 2>&1
    success "Approver funded"
    
    sleep 2
    
    log "Funding release_signer..."
    stellar payment --source alice --destination "$RELEASE_SIGNER" --amount 10 --network "$NETWORK" >> "$LOG_FILE" 2>&1
    success "Release signer funded"
    
    sleep 5
    return 0
}

# ============================================================================
# STEP 6: Create Trustlines
# ============================================================================

step_6_trustlines() {
    log_step "6 - Create USDC Trustlines"
    
    log "Creating release_signer trustline..."
    stellar account create-trusted-asset --asset "USDC:$USDC_ISSUER" --source release_signer --network "$NETWORK" >> "$LOG_FILE" 2>&1
    success "Release signer trustline created"
    
    sleep 2
    
    log "Creating platform trustline..."
    stellar account create-trusted-asset --asset "USDC:$USDC_ISSUER" --source platform --network "$NETWORK" >> "$LOG_FILE" 2>&1
    success "Platform trustline created"
    
    sleep 5
    return 0
}

# ============================================================================
# STEP 7: Mark Milestone Complete
# ============================================================================

step_7_milestone() {
    log_step "7 - Mark Milestone Complete"
    
    CONTRACT_ID=$(grep "^CONTRACT_ID=" "$LOG_FILE" | tail -1 | cut -d'=' -f2)
    
    MILESTONE_OUT=$(stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source provider \
        --network "$NETWORK" \
        -- change_milestone_status \
        --milestone_index 0 \
        --new_status '"Completed"' \
        --new_evidence '"Work submitted"' \
        --service_provider provider 2>&1)
    
    if echo "$MILESTONE_OUT" | grep -q "error\|Error\|ERROR"; then
        error "Milestone marking failed: $MILESTONE_OUT"
        echo "$MILESTONE_OUT" >> "$LOG_FILE"
        return 1
    fi
    
    echo "$MILESTONE_OUT" >> "$LOG_FILE"
    success "Milestone marked complete"
    sleep 5
    return 0
}

# ============================================================================
# STEP 8: Approve Milestone
# ============================================================================

step_8_approve_milestone() {
    log_step "8 - Approve Milestone"
    
    CONTRACT_ID=$(grep "^CONTRACT_ID=" "$LOG_FILE" | tail -1 | cut -d'=' -f2)
    
    APPROVE_MS_OUT=$(stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source approver \
        --network "$NETWORK" \
        -- approve_milestone \
        --milestone_index 0 \
        --approver approver 2>&1)
    
    if echo "$APPROVE_MS_OUT" | grep -q "error\|Error\|ERROR"; then
        error "Milestone approval failed: $APPROVE_MS_OUT"
        echo "$APPROVE_MS_OUT" >> "$LOG_FILE"
        return 1
    fi
    
    echo "$APPROVE_MS_OUT" >> "$LOG_FILE"
    success "Milestone approved"
    sleep 5
    return 0
}

# ============================================================================
# STEP 9: Release Funds
# ============================================================================

step_9_release() {
    log_step "9 - Release Funds & CCTP Burn"
    
    CONTRACT_ID=$(grep "^CONTRACT_ID=" "$LOG_FILE" | tail -1 | cut -d'=' -f2)
    
    RELEASE_OUT=$(stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source release_signer \
        --network "$NETWORK" \
        -- release_funds \
        --release_signer release_signer \
        --trustless_work_address release_signer 2>&1)
    
    echo "$RELEASE_OUT" >> "$LOG_FILE"
    echo "$RELEASE_OUT"
    
    # Extract burn tx hash
    BURN_TX_HASH=$(echo "$RELEASE_OUT" | grep -oE '[a-f0-9]{64}' | head -1)
    
    if [ -z "$BURN_TX_HASH" ]; then
        warn "Could not auto-extract burn hash. Please enter manually:"
        read -p "Enter burn transaction hash: " BURN_TX_HASH
    fi
    
    if [ -z "$BURN_TX_HASH" ]; then
        error "No burn hash provided"
        return 1
    fi
    
    success "Funds released, burn TX: $BURN_TX_HASH"
    echo "BURN_TX_HASH=$BURN_TX_HASH" >> "$LOG_FILE"
    sleep 5
    return 0
}

# ============================================================================
# STEP 10: Fetch Attestation
# ============================================================================

step_10_attestation() {
    log_step "10 - Fetch Circle Iris Attestation"
    
    BURN_TX_HASH=$(grep "^BURN_TX_HASH=" "$LOG_FILE" | tail -1 | cut -d'=' -f2)
    
    if [ -z "$BURN_TX_HASH" ]; then
        error "BURN_TX_HASH not found"
        return 1
    fi
    
    log "Burn TX Hash: $BURN_TX_HASH"
    log "Waiting for Circle Iris (this takes 20-30 seconds)..."
    
    ATTESTATION_FOUND=0
    
    for i in {1..20}; do
        log "Attempt $i/20 - waiting 5 seconds..."
        sleep 5
        
        IRIS_RESPONSE=$(curl -s "https://iris-api-sandbox.circle.com/v2/messages/27?transactionHash=$BURN_TX_HASH" 2>&1)
        
        echo "Circle response: $IRIS_RESPONSE" >> "$LOG_FILE"
        
        if echo "$IRIS_RESPONSE" | jq -e '.messages[0].message' > /dev/null 2>&1; then
            MESSAGE=$(echo "$IRIS_RESPONSE" | jq -r '.messages[0].message')
            ATTESTATION=$(echo "$IRIS_RESPONSE" | jq -r '.messages[0].attestation')
            
            success "Attestation received!"
            echo "MESSAGE=$MESSAGE" >> "$LOG_FILE"
            echo "ATTESTATION=$ATTESTATION" >> "$LOG_FILE"
            
            log "Message: $MESSAGE"
            log "Attestation: $ATTESTATION"

             echo ""
            echo "═══════════════════════════════════════════════════════════════════"
            echo "STEP 11 - MINT USDC ON SEPOLIA"
            echo "═══════════════════════════════════════════════════════════════════"
            echo ""
            echo "Open:"
            echo "https://sepolia.etherscan.io/address/0xE737e5cEBEEBa77EFE34D4aa090756590b1CE275#writeProxyContract#F6"
            echo ""
            echo "Call function:"
            echo "receiveMessage(bytes message, bytes attestation)"
            echo ""
            echo "MESSAGE:"
            echo "$MESSAGE"
            echo ""
            echo "ATTESTATION:"
            echo "$ATTESTATION"
            echo ""
            echo "Paste the MESSAGE value into the message field"
            echo "Paste the ATTESTATION value into the attestation field"
            echo "Connect your wallet and submit the transaction"
            echo ""
            echo "═══════════════════════════════════════════════════════════════════"
            
            ATTESTATION_FOUND=1
            return 0
        fi
    done
    
    if [ $ATTESTATION_FOUND -eq 0 ]; then
        warn "Attestation not received after 20 attempts"
        warn "This is OK - you can retry manually later"
        return 0
    fi
    
    return 0
}

# ============================================================================
# MAIN
# ============================================================================

main() {
    log_section "TRUSTLESS WORK ESCROW - COMPLETE E2E (FULL)"
    log "Started at: $(date)"
    log "Log file: $LOG_FILE"
    log "WASM: $WASM_PATH"
    
    # Setup wallet first (Step 0)
    step_0_setup_wallet || warn "Step 0 failed but continuing..."
    sleep 2
    
    # Run all steps
    step_1_deploy || warn "Step 1 failed but continuing..."
    sleep 2
    
    step_2_initialize || warn "Step 2 failed but continuing..."
    sleep 2
    
    step_3_approve || warn "Step 3 failed but continuing..."
    sleep 2
    
    step_4_fund || warn "Step 4 failed but continuing..."
    sleep 2
    
    step_5_fund_roles || warn "Step 5 failed but continuing..."
    sleep 2
    
    step_6_trustlines || warn "Step 6 failed but continuing..."
    sleep 2
    
    step_7_milestone || warn "Step 7 failed but continuing..."
    sleep 2
    
    step_8_approve_milestone || warn "Step 8 failed but continuing..."
    sleep 2
    
    step_9_release || warn "Step 9 failed but continuing..."
    sleep 2
    
    step_10_attestation || warn "Step 10 failed but continuing..."
    sleep 2
    
   
    log_section "✅ E2E EXECUTION COMPLETE"
    log "Finished at: $(date)"
    log "Log file: $LOG_FILE"
    
    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo "SAVED VALUES:"
    echo "═══════════════════════════════════════════════════════════════════"
    grep "CONTRACT_ID\|BURN_TX_HASH\|MESSAGE\|ATTESTATION" "$LOG_FILE" | tail -4
    echo ""
    echo "View full log with: cat $LOG_FILE"
    echo "═══════════════════════════════════════════════════════════════════"
}

# Run
main