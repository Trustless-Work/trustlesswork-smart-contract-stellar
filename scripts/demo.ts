/**
 * Demo: Trustless Work — Multi-Release Escrow
 *
 * Demonstrates (with on-chain state verified after each step):
 *  - Escrow with per-milestone amounts (3 + 4 + 3 = 10 USDC)
 *  - Multi-member roles (2 service providers, 2 approvers, 2 release signers)
 *  - Quorum flow: each milestone requires 2/2 votes to be approved
 *  - Partial releases: milestone 0 released first, then milestones 1 & 2 together
 *
 * ── Setup (first time) ────────────────────────────────────────────────────────
 *
 *  1. Install Rust and the WASM target (required to compile the contract)
 *       # macOS / Linux
 *       curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
 *
 *       # Windows — download and run the installer from:
 *       https://rustup.rs
 *
 *       # After installing Rust, add the WASM target:
 *       rustup target add wasm32v1-none
 *
 *  2. Install project dependencies
 *       bun install
 *
 *  3. Install Stellar CLI  →  https://developers.stellar.org/docs/tools/stellar-cli
 *       # macOS
 *       brew install stellar-cli
 *
 *       # Windows (winget)
 *       winget install --id Stellar.StellarCLI
 *
 *       # Windows (manual) — download the .exe from the releases page:
 *       https://github.com/stellar/stellar-cli/releases/latest
 *       Add the folder containing stellar.exe to your PATH environment variable.
 *
 *  4. Create a testnet account and fund it with XLM (automatic Friendbot)
 *       stellar keys generate --name <your-alias> --network testnet
 *
 *     This generates a keypair, saves it in the Stellar CLI local keystore,
 *     and funds it with testnet XLM via Friendbot.
 *     You can verify the balance at: https://stellar.expert/explorer/testnet
 *
 *  5. Get testnet USDC for that account
 *     The script deposits USDC from your account into the escrow, so you need
 *     at least 10 USDC on testnet. Two ways to get them:
 *
 *     Option A — Stellar Lab (swap XLM → USDC):
 *       https://lab.stellar.org/swap?network=testnet
 *       Connect your account (with your alias key) and swap XLM for USDC.
 *
 *     Option B — Classic Stellar Laboratory (path payment):
 *       https://laboratory.stellar.org/#txbuilder?network=test
 *       Build a "Path Payment" operation from XLM to USDC
 *       (testnet USDC issuer: GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5)
 *
 *  6. Create the .env file in this folder (scripts/) with your alias:
 *       echo "DEPLOYER=<your-alias>" > scripts/.env
 *
 *  7. Run the demo
 *       bun run demo.ts
 *
 * ──────────────────────────────────────────────────────────────────────────────
 */

import {
  Address,
  Asset,
  BASE_FEE,
  Contract,
  Keypair,
  Networks,
  Operation,
  rpc,
  TransactionBuilder,
  nativeToScVal,
  xdr,
} from "@stellar/stellar-sdk";
import { execSync } from "child_process";
import { randomBytes } from "crypto";
import { createWriteStream, mkdirSync } from "fs";
import { resolve } from "path";
import PDFDocument from "pdfkit";

// ── Log capture ───────────────────────────────────────────────────────────────

const capturedLogs: string[] = [];
const _origLog   = console.log.bind(console);
const _origError = console.error.bind(console);

console.log = (...args: unknown[]) => {
  const line = args.map((a) => (typeof a === "string" ? a : String(a))).join(" ");
  capturedLogs.push(line);
  _origLog(...args);
};
console.error = (...args: unknown[]) => {
  const line = args.map((a) => (typeof a === "string" ? a : String(a))).join(" ");
  capturedLogs.push("[ERROR] " + line);
  _origError(...args);
};

async function generateEvidencePDF(logs: string[]) {
  const evidenceDir = resolve(import.meta.dir, "evidence");
  mkdirSync(evidenceDir, { recursive: true });

  const ts       = new Date().toISOString().replace(/[:.]/g, "-");
  const filePath = resolve(evidenceDir, `demo-${ts}.pdf`);

  return new Promise<string>((resolveP, rejectP) => {
    const doc    = new PDFDocument({ margin: 50, size: "A4" });
    const stream = createWriteStream(filePath);
    doc.pipe(stream);

    // ── Header ──
    doc
      .fontSize(18)
      .font("Helvetica-Bold")
      .text("Trustless Work — Multi-Release Demo Evidence", { align: "center" });

    doc.moveDown(0.4);
    doc
      .fontSize(10)
      .font("Helvetica")
      .fillColor("#555555")
      .text(`Generated: ${new Date().toUTCString()}`, { align: "center" });

    doc.moveDown(1);
    const lineY = doc.y;
    doc.moveTo(50, lineY).lineTo(545, lineY).strokeColor("#cccccc").stroke();
    doc.moveDown(0.8);

    // ── Body: captured logs ──
    const EXPLORER_RE = /https?:\/\/[^\s]+/;

    for (const line of logs) {
      const match = line.match(EXPLORER_RE);

      if (match) {
        // Line containing a URL: render the whole line as a clickable link
        doc
          .font("Courier")
          .fontSize(8)
          .fillColor("#0055cc")
          .text(line, { link: match[0], underline: true });
        doc.fillColor("#000000");
      } else if (line.startsWith("===") || line.startsWith("[") || line.startsWith("  ►")) {
        // Section or step lines: bold
        doc.font("Courier-Bold").fontSize(8).fillColor("#000000").text(line);
      } else {
        doc.font("Courier").fontSize(8).fillColor("#000000").text(line);
      }
    }

    doc.end();
    stream.on("finish", () => resolveP(filePath));
    stream.on("error",  rejectP);
  });
}

// ── Configuration ─────────────────────────────────────────────────────────────

const NETWORK    = Networks.TESTNET;
const RPC_URL    = "https://soroban-testnet.stellar.org";
const HORIZON    = "https://horizon-testnet.stellar.org";
const USDC       = "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA";
const USDC_ASSET = new Asset("USDC", "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5");
const DEPLOYER   = process.env.DEPLOYER ?? "armandocodecr";
const ROOT       = resolve(import.meta.dir, "..");
const WASM       = `${ROOT}/target/wasm32v1-none/release/escrow.wasm`;
const EXPLORER   = "https://stellar.expert/explorer/testnet";

const server = new rpc.Server(RPC_URL);

// ── Roles ─────────────────────────────────────────────────────────────────────

const platform         = Keypair.random(); // admin + platform fee
const serviceProvider1 = Keypair.random(); // provider #1
const serviceProvider2 = Keypair.random(); // provider #2
const approver1        = Keypair.random(); // approver #1  ┐ quorum 2/2
const approver2        = Keypair.random(); // approver #2  ┘
const releaseSigner1   = Keypair.random(); // release signer #1  ┐ multi-signer
const releaseSigner2   = Keypair.random(); // release signer #2  ┘
const receiver         = Keypair.random(); // receives the payment
const disputeResolver  = Keypair.random(); // resolves disputes
const trustlessWork    = Keypair.random(); // TW protocol fee

const funder = Keypair.fromSecret(
  execSync(`stellar keys show ${DEPLOYER}`, { encoding: "utf8" }).trim()
);

// ── Basic helpers ─────────────────────────────────────────────────────────────

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

async function friendbot(...keys: Keypair[]) {
  await Promise.all(keys.map((k) =>
    fetch(`https://friendbot.stellar.org?addr=${k.publicKey()}`)
  ));
  await sleep(5000);
}

async function setupUsdcTrustline(...keys: Keypair[]) {
  await Promise.all(keys.map(async (kp) => {
    const account = await fetch(`${HORIZON}/accounts/${kp.publicKey()}`).then((r) => r.json());
    const tx = new TransactionBuilder(
      {
        accountId: () => kp.publicKey(),
        sequenceNumber: () => account.sequence,
        incrementSequenceNumber() { (this as any)._sequence = (BigInt((this as any)._sequence) + 1n).toString(); },
        _sequence: account.sequence,
      } as any,
      { fee: BASE_FEE, networkPassphrase: NETWORK }
    )
      .addOperation(Operation.changeTrust({ asset: USDC_ASSET, limit: "999999999" }))
      .setTimeout(30)
      .build();
    tx.sign(kp);
    await fetch(`${HORIZON}/transactions`, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({ tx: tx.toEnvelope().toXDR("base64") }),
    });
  }));
}

// Invokes a contract function and waits for confirmation — returns the tx hash
async function invoke(
  contractId: string,
  fn: string,
  args: xdr.ScVal[],
  signer: Keypair
): Promise<string> {
  const account = await server.getAccount(signer.publicKey());
  const tx = new TransactionBuilder(account, { fee: BASE_FEE, networkPassphrase: NETWORK })
    .addOperation(new Contract(contractId).call(fn, ...args))
    .setTimeout(60)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(sim)) throw new Error((sim as any).error);

  const builtTx = rpc.assembleTransaction(tx, sim as any).build();
  builtTx.sign(signer);

  const sent = await server.sendTransaction(builtTx);
  if (sent.status === "ERROR") throw new Error(`Error sending ${fn}`);

  // Polling via raw JSON-RPC (workaround: SDK v13 + Bun cannot parse the XDR response)
  for (let i = 0; i < 40; i++) {
    await sleep(2500);
    const body: any = await fetch(RPC_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getTransaction", params: { hash: sent.hash } }),
    }).then((r) => r.json());

    if (body.result?.status === "SUCCESS") return sent.hash;
    if (body.result?.status === "FAILED")  throw new Error(`Failed tx: ${fn}`);
  }
  throw new Error(`Timeout waiting for confirmation of: ${fn}`);
}

// ── ScVal builders ────────────────────────────────────────────────────────────

const str  = (v: string)      => nativeToScVal(v, { type: "string" });
const addr = (v: string)      => Address.fromString(v).toScVal();
const u32  = (v: number)      => nativeToScVal(v, { type: "u32" });
const i128 = (v: bigint)      => nativeToScVal(v, { type: "i128" });
const bool = (v: boolean)     => xdr.ScVal.scvBool(v);
const vec  = (v: xdr.ScVal[]) => xdr.ScVal.scvVec(v);
const opt  = (v: xdr.ScVal | null) => v ?? xdr.ScVal.scvVoid(); // Option<T>: Some(v) | None

// Soroban struct: fields sorted alphabetically (contracttype requirement)
const scStruct = (fields: [string, xdr.ScVal][]) =>
  xdr.ScVal.scvMap(
    [...fields]
      .sort((a, b) => a[0].localeCompare(b[0]))
      .map(([k, v]) => new xdr.ScMapEntry({ key: xdr.ScVal.scvSymbol(k), val: v }))
  );

// ── On-chain contract state query ─────────────────────────────────────────────

// Simulates a read function and returns the resulting ScVal
async function simulateView(
  contractId: string,
  fn: string,
  args: xdr.ScVal[] = []
): Promise<xdr.ScVal> {
  const account = await server.getAccount(funder.publicKey());
  const tx = new TransactionBuilder(account, { fee: BASE_FEE, networkPassphrase: NETWORK })
    .addOperation(new Contract(contractId).call(fn, ...args))
    .setTimeout(60)
    .build();

  const body: any = await fetch(RPC_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0", id: 1,
      method: "simulateTransaction",
      params: { transaction: tx.toEnvelope().toXDR("base64") },
    }),
  }).then((r) => r.json());

  const resultXdr = body.result?.results?.[0]?.xdr;
  if (!resultXdr) throw new Error(`simulateView "${fn}": no XDR result`);
  return xdr.ScVal.fromXDR(resultXdr, "base64");
}

// Navigates a scvMap by symbol key
function mapGet(val: xdr.ScVal, key: string): xdr.ScVal {
  for (const entry of val.map()) {
    if (entry.key().sym().toString() === key) return entry.val();
  }
  throw new Error(`Field "${key}" not found in map`);
}

function scvStr(v: xdr.ScVal): string   { return v.str().toString(); }
function scvBool(v: xdr.ScVal): boolean { return v.b(); }
function scvU32(v: xdr.ScVal): number   { return v.u32(); }
function scvVec(v: xdr.ScVal): xdr.ScVal[] { return v.vec() ?? []; }

function i128ToBigInt(v: xdr.ScVal): bigint {
  const parts = v.i128();
  const hi = BigInt(parts.hi().toString());
  const lo = BigInt(parts.lo().toString());
  return (hi << 64n) | lo;
}

function formatUsdc(raw: bigint): string {
  const whole = raw / 10_000_000n;
  const frac  = String(raw % 10_000_000n).padStart(7, "0");
  return `${whole}.${frac} USDC`;
}

// ── On-chain state panels ─────────────────────────────────────────────────────

const DIV = "─".repeat(60);

async function showMilestones(contractId: string, label: string) {
  const escrow     = await simulateView(contractId, "get_escrow");
  const milestones = scvVec(mapGet(escrow, "milestones"));

  console.log(`\n  ┌─ on-chain: ${label} ${DIV.slice(label.length + 10)}`);
  console.log(`  │  milestones:`);

  for (let i = 0; i < milestones.length; i++) {
    const m         = milestones[i];
    const status    = scvStr(mapGet(m, "status")).padEnd(10);
    const evidence  = scvStr(mapGet(m, "evidence"));
    const released  = scvBool(mapGet(m, "released"));
    const amount    = i128ToBigInt(mapGet(m, "amount"));
    const approvals = mapGet(m, "approvals");
    const count     = scvU32(mapGet(approvals, "approval_count"));
    const target    = scvU32(mapGet(approvals, "target"));
    const quorum    = count >= target ? `${count}/${target} ✓` : `${count}/${target}`;
    const rel       = released ? " released ✓" : "";
    const ev        = evidence ? `  "${evidence.slice(0, 32)}${evidence.length > 32 ? "…" : ""}"` : "";
    console.log(`  │    [${i}] ${status}  ${formatUsdc(amount).padEnd(16)}  quorum: ${quorum.padEnd(6)}${rel}${ev}`);
  }

  console.log(`  └${DIV}\n`);
}

async function showContractBalance(contractId: string) {
  const raw = i128ToBigInt(await simulateView(USDC, "balance", [addr(contractId)]));
  console.log(`\n  ┌─ on-chain: contract balance ${"─".repeat(31)}`);
  console.log(`  │  contract USDC:  ${formatUsdc(raw)}`);
  console.log(`  └${DIV}\n`);
}

async function showReleaseBalances(contractId: string, label: string) {
  const balances = await Promise.all([
    simulateView(USDC, "balance", [addr(contractId)]),
    simulateView(USDC, "balance", [addr(receiver.publicKey())]),
    simulateView(USDC, "balance", [addr(platform.publicKey())]),
    simulateView(USDC, "balance", [addr(trustlessWork.publicKey())]),
  ]);

  const [contractBal, receiverBal, platformBal, twBal] = balances.map(i128ToBigInt);

  console.log(`\n  ┌─ on-chain: ${label} ${DIV.slice(label.length + 10)}`);
  console.log(`  │  contract:       ${formatUsdc(contractBal)}`);
  console.log(`  │  receiver:       ${formatUsdc(receiverBal)}`);
  console.log(`  │  platform (fee): ${formatUsdc(platformBal)}`);
  console.log(`  │  trustlessWork:  ${formatUsdc(twBal)}`);
  console.log(`  └${DIV}\n`);
}

// ── Main ──────────────────────────────────────────────────────────────────────

async function main() {
  const QUORUM = 2;

  // Per-milestone amounts: 3 + 4 + 3 = 10 USDC (7 decimals)
  const AMOUNT_M0    = 3_0000000n;
  const AMOUNT_M1    = 4_0000000n;
  const AMOUNT_M2    = 3_0000000n;
  const AMOUNT_TOTAL = AMOUNT_M0 + AMOUNT_M1 + AMOUNT_M2;

  // ── Funder account check ──────────────────────────────────────────────────
  console.log(`\nVerifying funder account (DEPLOYER=${DEPLOYER})...`);

  // 1. Does the account exist on testnet?
  try {
    await server.getAccount(funder.publicKey());
  } catch {
    console.error(`
  ERROR: account "${DEPLOYER}" (${funder.publicKey()}) does not exist on testnet.

  Create and fund it with XLM by running:
    stellar keys generate --name ${DEPLOYER} --network testnet

  If you already have the keypair but no funds:
    curl "https://friendbot.stellar.org?addr=${funder.publicKey()}"
`);
    process.exit(1);
  }

  // 2. Does it have enough USDC? (requires trustline + balance >= 10 USDC)
  let funderUsdc = 0n;
  try {
    funderUsdc = i128ToBigInt(await simulateView(USDC, "balance", [addr(funder.publicKey())]));
  } catch {
    // Simulation may fail if there is no trustline
  }

  if (funderUsdc < AMOUNT_TOTAL) {
    console.error(`
  ERROR: account "${DEPLOYER}" needs at least 10 USDC on testnet.
  Current balance: ${formatUsdc(funderUsdc)}

  How to get testnet USDC:
    • Stellar Lab (swap XLM → USDC):
        https://lab.stellar.org/swap?network=testnet
    • Or use Stellar Laboratory (path payment from XLM):
        https://laboratory.stellar.org/#txbuilder?network=test
      Testnet USDC issuer: GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5
`);
    process.exit(1);
  }

  console.log(`  ok — balance: ${formatUsdc(funderUsdc)}\n`);

  // ── Demo start ────────────────────────────────────────────────────────────
  console.log("=== Trustless Work — Multi-Release Demo ===\n");
  console.log("Roles:");
  console.log(`  platform          ${platform.publicKey()}  (admin + platform)`);
  console.log(`  serviceProvider1  ${serviceProvider1.publicKey()}`);
  console.log(`  serviceProvider2  ${serviceProvider2.publicKey()}`);
  console.log(`  approver1         ${approver1.publicKey()}  ─┐ quorum ${QUORUM}/2`);
  console.log(`  approver2         ${approver2.publicKey()}  ─┘`);
  console.log(`  releaseSigner1    ${releaseSigner1.publicKey()}  ─┐ multi-signer`);
  console.log(`  releaseSigner2    ${releaseSigner2.publicKey()}  ─┘`);
  console.log(`  receiver          ${receiver.publicKey()}`);
  console.log(`  funder (USDC)     ${funder.publicKey()}`);
  console.log(`\n  Milestone amounts: [0] ${formatUsdc(AMOUNT_M0)}  [1] ${formatUsdc(AMOUNT_M1)}  [2] ${formatUsdc(AMOUNT_M2)}  total: ${formatUsdc(AMOUNT_TOTAL)}`);

  // [1/6] Fund with Friendbot
  console.log("\n[1/6] Funding accounts with Friendbot...");
  await friendbot(
    platform, serviceProvider1, serviceProvider2,
    approver1, approver2,
    releaseSigner1, releaseSigner2,
    receiver, disputeResolver, trustlessWork
  );
  console.log("      ok");

  // [2/6] USDC trustlines for accounts that will receive funds in release_funds
  console.log("\n[2/6] Setting up USDC trustlines (receiver / trustlessWork / platform)...");
  await setupUsdcTrustline(receiver, trustlessWork, platform);
  console.log("      ok");

  // [3/6] Compile
  console.log("\n[3/6] Compiling contract...");
  execSync("cargo build --target wasm32v1-none --release --quiet", { cwd: ROOT, stdio: "inherit" });
  console.log("      ok");

  // [4/6] Deploy
  console.log("\n[4/6] Deploying contract to testnet...");
  const wasmHash = execSync(
    `stellar contract upload --wasm ${WASM} --source ${DEPLOYER} --network testnet`,
    { encoding: "utf8" }
  ).trim();

  const contractId = execSync(
    `stellar contract deploy \
      --wasm ${WASM} \
      --source ${DEPLOYER} \
      --network testnet \
      --salt ${randomBytes(32).toString("hex")} \
      -- \
      --admin ${platform.publicKey()} \
      --approved-wasm-hash ${wasmHash}`,
    { encoding: "utf8" }
  ).trim();
  console.log(`      ok  ${EXPLORER}/contract/${contractId}`);

  // ── Escrow structure ──────────────────────────────────────────────────────
  // In multi-release, each Milestone carries its own amount, dispute, and released flag.

  const milestone = (desc: string, amount: bigint) =>
    scStruct([
      ["amount",      i128(amount)],
      ["approvals",   scStruct([
        ["approval_count", u32(0)],
        ["approvers",      vec([])],
        ["target",         u32(QUORUM)],
      ])],
      ["description", str(desc)],
      ["dispute",     scStruct([
        ["is_disputed", bool(false)],
        ["reason",      str("")],
        ["resolved",    bool(false)],
      ])],
      ["evidence",    str("")],
      ["released",    bool(false)],
      ["status",      str("Pending")],
    ]);

  const escrow = scStruct([
    ["description",   str("Demo: multi-release freelance contract with quorum")],
    ["engagement_id", str("demo-multi-release-001")],
    ["milestones", vec([
      milestone("UI Design",                AMOUNT_M0),
      milestone("Backend implementation",   AMOUNT_M1),
      milestone("QA and production deploy", AMOUNT_M2),
    ])],
    ["platform_fee",  u32(300)],   // 3%
    ["receiver_memo", u32(0)],
    ["roles", scStruct([
      ["admin",             addr(platform.publicKey())],
      ["approvers",         vec([addr(approver1.publicKey()), addr(approver2.publicKey())])],
      ["dispute_resolvers", vec([addr(disputeResolver.publicKey())])],
      ["observers",         vec([])],
      ["platform",          addr(platform.publicKey())],
      ["receiver",          addr(receiver.publicKey())],
      ["release_signers",   vec([addr(releaseSigner1.publicKey()), addr(releaseSigner2.publicKey())])],
      ["service_providers", vec([addr(serviceProvider1.publicKey()), addr(serviceProvider2.publicKey())])],
    ])],
    ["title",     str("Demo Multi-Release Escrow — quorum and partial releases")],
    ["trustline", scStruct([["address", addr(USDC)]])],
  ]);

  // ── Escrow flow ───────────────────────────────────────────────────────────
  console.log(`\n[5/6] Escrow flow (3 milestones, quorum ${QUORUM}/2, partial releases)\n`);

  // initialize_escrow
  console.log("  ► initialize_escrow   (platform)");
  const h1 = await invoke(contractId, "initialize_escrow", [escrow], platform);
  console.log(`    ${EXPLORER}/tx/${h1}`);
  await showMilestones(contractId, "post initialize_escrow");

  // fund_escrow — deposit total amount (sum of all milestone amounts)
  console.log(`  ► fund_escrow         (funder deposits ${formatUsdc(AMOUNT_TOTAL)})`);
  const h2 = await invoke(contractId, "fund_escrow", [addr(funder.publicKey()), escrow, i128(AMOUNT_TOTAL)], funder);
  console.log(`    ${EXPLORER}/tx/${h2}`);
  await showContractBalance(contractId);

  // change_milestone_status — batch all 3 milestones in a single tx
  console.log("  ► change_milestone_status  (serviceProvider1 — batch of 3 milestones)");
  const updates = [
    { index: 0, evidence: "PR #42 merged — design finalized",     status: "Completed" },
    { index: 1, evidence: "API deployed to staging, tests passed", status: "Completed" },
    { index: 2, evidence: "100% coverage, prod deploy verified",   status: "Completed" },
  ];
  const h3 = await invoke(contractId, "change_milestone_status", [
    vec(updates.map((u) =>
      scStruct([
        ["milestone_index", u32(u.index)],
        ["new_evidence",    opt(str(u.evidence))],  // Option<String>: Some(evidence)
        ["new_status",      str(u.status)],
      ])
    )),
    addr(serviceProvider1.publicKey()),
  ], serviceProvider1);
  console.log(`    ${EXPLORER}/tx/${h3}`);
  await showMilestones(contractId, "post change_milestone_status");

  // approve_milestones — approver1 votes on all 3 (quorum incomplete)
  console.log("  ► approve_milestones  (approver1 — votes on 3 milestones)");
  const h4 = await invoke(contractId, "approve_milestones",
    [vec([u32(0), u32(1), u32(2)]), addr(approver1.publicKey())],
    approver1
  );
  console.log(`    ${EXPLORER}/tx/${h4}`);
  await showMilestones(contractId, "post approve_milestones [approver1]");

  // approve_milestones — approver2 votes on all 3 (quorum reached)
  console.log("  ► approve_milestones  (approver2 — votes on 3 milestones)");
  const h5 = await invoke(contractId, "approve_milestones",
    [vec([u32(0), u32(1), u32(2)]), addr(approver2.publicKey())],
    approver2
  );
  console.log(`    ${EXPLORER}/tx/${h5}`);
  await showMilestones(contractId, "post approve_milestones [approver2]");

  // release_funds [0] — partial release: milestone 0 only (3 USDC)
  console.log(`  ► release_funds [0]   (releaseSigner1 — releases milestone 0: ${formatUsdc(AMOUNT_M0)})`);
  const h6 = await invoke(contractId, "release_funds",
    [addr(releaseSigner1.publicKey()), addr(trustlessWork.publicKey()), vec([u32(0)])],
    releaseSigner1
  );
  console.log(`    ${EXPLORER}/tx/${h6}`);
  await showMilestones(contractId, "post release_funds [0]");
  await showReleaseBalances(contractId, "balances after release [0]");

  // release_funds [1, 2] — final release: milestones 1 and 2 together (7 USDC)
  console.log(`  ► release_funds [1,2] (releaseSigner1 — releases milestones 1 & 2: ${formatUsdc(AMOUNT_M1 + AMOUNT_M2)})`);
  const h7 = await invoke(contractId, "release_funds",
    [addr(releaseSigner1.publicKey()), addr(trustlessWork.publicKey()), vec([u32(1), u32(2)])],
    releaseSigner1
  );
  console.log(`    ${EXPLORER}/tx/${h7}`);
  await showMilestones(contractId, "post release_funds [1,2]");
  await showReleaseBalances(contractId, "final balances");

  // ── Summary ───────────────────────────────────────────────────────────────
  console.log("[6/6] Summary\n");
  console.log(`  Contract:                  ${EXPLORER}/contract/${contractId}`);
  console.log(`  initialize_escrow:         ${EXPLORER}/tx/${h1}`);
  console.log(`  fund_escrow:               ${EXPLORER}/tx/${h2}`);
  console.log(`  change_milestone_status:   ${EXPLORER}/tx/${h3}`);
  console.log(`  approve_milestones [1]:    ${EXPLORER}/tx/${h4}`);
  console.log(`  approve_milestones [2]:    ${EXPLORER}/tx/${h5}`);
  console.log(`  release_funds [0]:         ${EXPLORER}/tx/${h6}`);
  console.log(`  release_funds [1,2]:       ${EXPLORER}/tx/${h7}`);
  console.log("\n=== Flow complete ===");

  // ── Generate evidence PDF ─────────────────────────────────────────────────
  _origLog("\nGenerating evidence PDF...");
  const pdfPath = await generateEvidencePDF(capturedLogs);
  _origLog(`  PDF saved at: ${pdfPath}`);
}

main().catch(async (e) => {
  console.error("\n ERROR:", e.message);
  _origLog("\nGenerating evidence PDF (with error)...");
  try {
    const pdfPath = await generateEvidencePDF(capturedLogs);
    _origLog(`  PDF saved at: ${pdfPath}`);
  } catch (pdfErr) {
    _origLog("  Could not generate PDF:", (pdfErr as Error).message);
  }
  process.exit(1);
});
