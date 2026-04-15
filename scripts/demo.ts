/**
 * Demo: Trustless Work — nuevas funcionalidades
 *
 * Demuestra (con estado on-chain verificado tras cada paso):
 *  - Batch update de 3 milestones en una sola transacción
 *  - Roles multi-miembro (2 service providers, 2 approvers, 2 release signers)
 *  - Flujo de quórum: cada milestone requiere 2/2 votos para quedar aprobado
 *
 * ── Setup (primera vez) ────────────────────────────────────────────────────────
 *
 *  1. Instalar dependencias del proyecto
 *       bun install
 *
 *  2. Instalar Stellar CLI  →  https://developers.stellar.org/docs/tools/stellar-cli
 *       # macOS
 *       brew install stellar-cli
 *
 *  3. Crear una cuenta en testnet y fondearla con XLM (Friendbot automático)
 *       stellar keys generate --name <tu-alias> --network testnet
 *
 *     Esto genera un keypair, lo guarda en el keystore local de Stellar CLI
 *     y lo fondea con XLM de testnet via Friendbot.
 *     Puedes verificar el saldo en: https://stellar.expert/explorer/testnet
 *
 *  4. Obtener USDC de testnet para esa cuenta
 *     El script deposita USDC desde tu cuenta al escrow, así que necesitas al
 *     menos 10 USDC en testnet. Hay dos formas de conseguirlos:
 *
 *     Opción A — Stellar Lab (swap XLM → USDC):
 *       https://lab.stellar.org/swap?network=testnet
 *       Conecta tu cuenta (con la clave de tu alias) y haz swap de XLM por USDC.
 *
 *     Opción B — Stellar Laboratory clásico (path payment):
 *       https://laboratory.stellar.org/#txbuilder?network=test
 *       Construye una operación "Path Payment" de XLM a USDC
 *       (emisor USDC testnet: GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5)
 *
 *  5. Crear el archivo .env en esta carpeta (scripts/) con tu alias:
 *       echo "DEPLOYER=<tu-alias>" > scripts/.env
 *
 *  6. Ejecutar el demo
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

// ── Captura de logs ───────────────────────────────────────────────────────────

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

    // ── Encabezado ──
    doc
      .fontSize(18)
      .font("Helvetica-Bold")
      .text("Trustless Work — Demo Evidence", { align: "center" });

    doc.moveDown(0.4);
    doc
      .fontSize(10)
      .font("Helvetica")
      .fillColor("#555555")
      .text(`Generado: ${new Date().toUTCString()}`, { align: "center" });

    doc.moveDown(1);
    const lineY = doc.y;
    doc.moveTo(50, lineY).lineTo(545, lineY).strokeColor("#cccccc").stroke();
    doc.moveDown(0.8);

    // ── Cuerpo: logs capturados ──
    const EXPLORER_RE = /https?:\/\/[^\s]+/;

    for (const line of logs) {
      const match = line.match(EXPLORER_RE);

      if (match) {
        // Línea que contiene una URL: renderiza toda la línea como link clicable
        doc
          .font("Courier")
          .fontSize(8)
          .fillColor("#0055cc")
          .text(line, { link: match[0], underline: true });
        doc.fillColor("#000000");
      } else if (line.startsWith("===") || line.startsWith("[") || line.startsWith("  ►")) {
        // Líneas de sección o paso: negrita
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

// ── Configuración ─────────────────────────────────────────────────────────────

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
const serviceProvider1 = Keypair.random(); // proveedor #1
const serviceProvider2 = Keypair.random(); // proveedor #2
const approver1        = Keypair.random(); // aprobador #1  ┐ quórum 2/2
const approver2        = Keypair.random(); // aprobador #2  ┘
const releaseSigner1   = Keypair.random(); // release signer #1  ┐ multi-signer
const releaseSigner2   = Keypair.random(); // release signer #2  ┘
const receiver         = Keypair.random(); // recibe el pago
const disputeResolver  = Keypair.random(); // resuelve disputas
const trustlessWork    = Keypair.random(); // fee de protocolo TW

const funder = Keypair.fromSecret(
  execSync(`stellar keys show ${DEPLOYER}`, { encoding: "utf8" }).trim()
);

// ── Helpers básicos ───────────────────────────────────────────────────────────

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

// Invoca una función de contrato y espera confirmación — devuelve el tx hash
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
  if (sent.status === "ERROR") throw new Error(`Error enviando ${fn}`);

  // Polling via raw JSON-RPC (workaround: SDK v13 + Bun no puede parsear la respuesta XDR)
  for (let i = 0; i < 40; i++) {
    await sleep(2500);
    const body: any = await fetch(RPC_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getTransaction", params: { hash: sent.hash } }),
    }).then((r) => r.json());

    if (body.result?.status === "SUCCESS") return sent.hash;
    if (body.result?.status === "FAILED")  throw new Error(`Tx fallida: ${fn}`);
  }
  throw new Error(`Timeout esperando confirmación de: ${fn}`);
}

// ── ScVal builders ────────────────────────────────────────────────────────────

const str  = (v: string)      => nativeToScVal(v, { type: "string" });
const addr = (v: string)      => Address.fromString(v).toScVal();
const u32  = (v: number)      => nativeToScVal(v, { type: "u32" });
const i128 = (v: bigint)      => nativeToScVal(v, { type: "i128" });
const bool = (v: boolean)     => xdr.ScVal.scvBool(v);
const vec  = (v: xdr.ScVal[]) => xdr.ScVal.scvVec(v);

// Struct de Soroban: campos ordenados alfabéticamente (requisito de contracttype)
const scStruct = (fields: [string, xdr.ScVal][]) =>
  xdr.ScVal.scvMap(
    [...fields]
      .sort((a, b) => a[0].localeCompare(b[0]))
      .map(([k, v]) => new xdr.ScMapEntry({ key: xdr.ScVal.scvSymbol(k), val: v }))
  );

// ── Consulta on-chain del estado del contrato ─────────────────────────────────

// Simula una función de lectura y devuelve el ScVal resultado
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
  if (!resultXdr) throw new Error(`simulateView "${fn}": sin resultado XDR`);
  return xdr.ScVal.fromXDR(resultXdr, "base64");
}

// Navega un scvMap por clave símbolo
function mapGet(val: xdr.ScVal, key: string): xdr.ScVal {
  for (const entry of val.map()) {
    if (entry.key().sym().toString() === key) return entry.val();
  }
  throw new Error(`Campo "${key}" no encontrado en el mapa`);
}

function scvStr(v: xdr.ScVal): string  { return v.str().toString(); }
function scvBool(v: xdr.ScVal): boolean { return v.b(); }
function scvU32(v: xdr.ScVal): number  { return v.u32(); }
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

// ── Paneles de estado on-chain ────────────────────────────────────────────────

const DIV = "─".repeat(58);

async function showMilestones(contractId: string, label: string) {
  const escrow     = await simulateView(contractId, "get_escrow");
  const released   = scvBool(mapGet(escrow, "released"));
  const milestones = scvVec(mapGet(escrow, "milestones"));

  console.log(`\n  ┌─ on-chain: ${label} ${DIV.slice(label.length + 10)}`);
  console.log(`  │  released: ${released}`);
  console.log(`  │  milestones:`);

  for (let i = 0; i < milestones.length; i++) {
    const m         = milestones[i];
    const status    = scvStr(mapGet(m, "status")).padEnd(10);
    const evidence  = scvStr(mapGet(m, "evidence"));
    const approvals = mapGet(m, "approvals");
    const count     = scvU32(mapGet(approvals, "approval_count"));
    const target    = scvU32(mapGet(approvals, "target"));
    const quorum    = count >= target ? `${count}/${target} ✓` : `${count}/${target}`;
    const ev        = evidence ? `  "${evidence.slice(0, 38)}${evidence.length > 38 ? "…" : ""}"` : "";
    console.log(`  │    [${i}] ${status}  quórum: ${quorum.padEnd(6)}${ev}`);
  }

  console.log(`  └${DIV}\n`);
}

async function showContractBalance(contractId: string) {
  const raw = i128ToBigInt(await simulateView(USDC, "balance", [addr(contractId)]));
  console.log(`\n  ┌─ on-chain: saldo del contrato ${"─".repeat(27)}`);
  console.log(`  │  contrato USDC:  ${formatUsdc(raw)}`);
  console.log(`  └${DIV}\n`);
}

async function showReleaseBalances(contractId: string) {
  const escrow   = await simulateView(contractId, "get_escrow");
  const released = scvBool(mapGet(escrow, "released"));

  const balances = await Promise.all([
    simulateView(USDC, "balance", [addr(contractId)]),
    simulateView(USDC, "balance", [addr(receiver.publicKey())]),
    simulateView(USDC, "balance", [addr(platform.publicKey())]),
    simulateView(USDC, "balance", [addr(trustlessWork.publicKey())]),
  ]);

  const [contractBal, receiverBal, platformBal, twBal] = balances.map(i128ToBigInt);

  console.log(`\n  ┌─ on-chain: post release_funds ${"─".repeat(26)}`);
  console.log(`  │  released:       ${released}`);
  console.log(`  │  contrato:       ${formatUsdc(contractBal)}`);
  console.log(`  │  receiver:       ${formatUsdc(receiverBal)}`);
  console.log(`  │  platform (fee): ${formatUsdc(platformBal)}`);
  console.log(`  │  trustlessWork:  ${formatUsdc(twBal)}`);
  console.log(`  └${DIV}\n`);
}

// ── Main ──────────────────────────────────────────────────────────────────────

async function main() {
  const QUORUM = 2;
  const AMOUNT = 10_0000000n; // 10 USDC (7 decimales)

  // ── Verificación de la cuenta funder ─────────────────────────────────────
  console.log(`\nVerificando cuenta funder (DEPLOYER=${DEPLOYER})...`);

  // 1. ¿Existe la cuenta en testnet?
  try {
    await server.getAccount(funder.publicKey());
  } catch {
    console.error(`
  ERROR: la cuenta "${DEPLOYER}" (${funder.publicKey()}) no existe en testnet.

  Créala y fóndela con XLM ejecutando:
    stellar keys generate --name ${DEPLOYER} --network testnet

  Si ya tienes el keypair pero sin fondos:
    curl "https://friendbot.stellar.org?addr=${funder.publicKey()}"
`);
    process.exit(1);
  }

  // 2. ¿Tiene USDC suficiente? (requiere trustline + saldo >= 10 USDC)
  let funderUsdc = 0n;
  try {
    funderUsdc = i128ToBigInt(await simulateView(USDC, "balance", [addr(funder.publicKey())]));
  } catch {
    // La simulación puede fallar si no tiene trustline
  }

  if (funderUsdc < AMOUNT) {
    console.error(`
  ERROR: la cuenta "${DEPLOYER}" necesita al menos 10 USDC en testnet.
  Saldo actual: ${formatUsdc(funderUsdc)}

  Cómo obtener USDC de testnet:
    • Stellar Lab (swap XLM → USDC):
        https://lab.stellar.org/swap?network=testnet
    • O usa Stellar Laboratory (path payment desde XLM):
        https://laboratory.stellar.org/#txbuilder?network=test
      Emisor USDC testnet: GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5
`);
    process.exit(1);
  }

  console.log(`  ok — saldo: ${formatUsdc(funderUsdc)}\n`);

  // ── Inicio del demo ───────────────────────────────────────────────────────
  console.log("=== Trustless Work — Demo: nuevas funcionalidades ===\n");
  console.log("Roles:");
  console.log(`  platform          ${platform.publicKey()}  (admin + plataforma)`);
  console.log(`  serviceProvider1  ${serviceProvider1.publicKey()}`);
  console.log(`  serviceProvider2  ${serviceProvider2.publicKey()}`);
  console.log(`  approver1         ${approver1.publicKey()}  ─┐ quórum ${QUORUM}/2`);
  console.log(`  approver2         ${approver2.publicKey()}  ─┘`);
  console.log(`  releaseSigner1    ${releaseSigner1.publicKey()}  ─┐ multi-signer`);
  console.log(`  releaseSigner2    ${releaseSigner2.publicKey()}  ─┘`);
  console.log(`  receiver          ${receiver.publicKey()}`);
  console.log(`  funder (USDC)     ${funder.publicKey()}`);

  // [1] Fondear con Friendbot
  console.log("\n[1/6] Fondeando cuentas con Friendbot...");
  await friendbot(
    platform, serviceProvider1, serviceProvider2,
    approver1, approver2,
    releaseSigner1, releaseSigner2,
    receiver, disputeResolver, trustlessWork
  );
  console.log("      ok");

  // [2] Trustlines USDC para los que recibirán fondos en release_funds
  console.log("\n[2/6] Configurando trustlines USDC (receiver / trustlessWork / platform)...");
  await setupUsdcTrustline(receiver, trustlessWork, platform);
  console.log("      ok");

  // [3] Compilar
  console.log("\n[3/6] Compilando contrato...");
  execSync("cargo build --target wasm32v1-none --release --quiet", { cwd: ROOT, stdio: "inherit" });
  console.log("      ok");

  // [4] Desplegar
  console.log("\n[4/6] Desplegando contrato en testnet...");
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

  // ── Estructura del escrow ─────────────────────────────────────────────────

  const milestone = (desc: string) =>
    scStruct([
      ["approvals", scStruct([
        ["approval_count", u32(0)],
        ["approvers",      vec([])],
        ["target",         u32(QUORUM)],
      ])],
      ["description", str(desc)],
      ["evidence",    str("")],
      ["status",      str("Pending")],
    ]);

  const escrow = scStruct([
    ["amount",        i128(AMOUNT)],
    ["description",   str("Demo: contrato de trabajo freelance con quórum")],
    ["dispute",       scStruct([
      ["is_disputed", bool(false)],
      ["reason",      str("")],
      ["resolved",    bool(false)],
    ])],
    ["engagement_id", str("demo-quorum-001")],
    ["milestones", vec([
      milestone("Diseño de UI"),
      milestone("Implementación del backend"),
      milestone("QA y deploy a producción"),
    ])],
    ["platform_fee",  u32(300)],   // 3%
    ["receiver_memo", u32(0)],
    ["released",      bool(false)],
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
    ["title",     str("Demo Escrow — quórum y multi-roles")],
    ["trustline", scStruct([["address", addr(USDC)]])],
  ]);

  // ── Flujo del escrow ──────────────────────────────────────────────────────
  console.log(`\n[5/6] Flujo del escrow (3 milestones, quórum ${QUORUM}/2, multi-roles)\n`);

  // initialize_escrow
  console.log("  ► initialize_escrow   (platform)");
  const h1 = await invoke(contractId, "initialize_escrow", [escrow], platform);
  console.log(`    ${EXPLORER}/tx/${h1}`);
  await showMilestones(contractId, "post initialize_escrow");

  // fund_escrow
  console.log("  ► fund_escrow         (funder deposita 10 USDC)");
  const h2 = await invoke(contractId, "fund_escrow", [addr(funder.publicKey()), escrow, i128(AMOUNT)], funder);
  console.log(`    ${EXPLORER}/tx/${h2}`);
  await showContractBalance(contractId);

  // change_milestone_status — batch de los 3 milestones en una sola tx
  console.log("  ► change_milestone_status  (serviceProvider1 — batch de 3 milestones)");
  const updates = [
    { index: 0, evidence: "PR #42 mergeado — diseño finalizado",     status: "Completed" },
    { index: 1, evidence: "API desplegada en staging, tests pasados", status: "Completed" },
    { index: 2, evidence: "100% coverage, deploy a prod verificado",  status: "Completed" },
  ];
  const h3 = await invoke(contractId, "change_milestone_status", [
    vec(updates.map((u) =>
      scStruct([
        ["milestone_index", u32(u.index)],
        ["new_evidence",    str(u.evidence)],
        ["new_status",      str(u.status)],
      ])
    )),
    addr(serviceProvider1.publicKey()),
  ], serviceProvider1);
  console.log(`    ${EXPLORER}/tx/${h3}`);
  await showMilestones(contractId, "post change_milestone_status");

  // approve_milestones — approver1 vota los 3 (quórum incompleto)
  console.log("  ► approve_milestones  (approver1 — vota los 3 milestones)");
  const h4 = await invoke(contractId, "approve_milestones",
    [vec([u32(0), u32(1), u32(2)]), addr(approver1.publicKey())],
    approver1
  );
  console.log(`    ${EXPLORER}/tx/${h4}`);
  await showMilestones(contractId, "post approve_milestones [approver1]");

  // approve_milestones — approver2 vota los 3 (quórum alcanzado)
  console.log("  ► approve_milestones  (approver2 — vota los 3 milestones)");
  const h5 = await invoke(contractId, "approve_milestones",
    [vec([u32(0), u32(1), u32(2)]), addr(approver2.publicKey())],
    approver2
  );
  console.log(`    ${EXPLORER}/tx/${h5}`);
  await showMilestones(contractId, "post approve_milestones [approver2]");

  // release_funds — cualquiera de los release signers puede liberar
  console.log("  ► release_funds       (releaseSigner1)");
  const h6 = await invoke(contractId, "release_funds",
    [addr(releaseSigner1.publicKey()), addr(trustlessWork.publicKey())],
    releaseSigner1
  );
  console.log(`    ${EXPLORER}/tx/${h6}`);
  await showReleaseBalances(contractId);

  // ── Resumen ───────────────────────────────────────────────────────────────
  console.log("[6/6] Resumen\n");
  console.log(`  Contrato:                ${EXPLORER}/contract/${contractId}`);
  console.log(`  initialize_escrow:       ${EXPLORER}/tx/${h1}`);
  console.log(`  fund_escrow:             ${EXPLORER}/tx/${h2}`);
  console.log(`  change_milestone_status: ${EXPLORER}/tx/${h3}`);
  console.log(`  approve_milestones [1]:  ${EXPLORER}/tx/${h4}`);
  console.log(`  approve_milestones [2]:  ${EXPLORER}/tx/${h5}`);
  console.log(`  release_funds:           ${EXPLORER}/tx/${h6}`);
  console.log("\n=== Flujo completo ===");

  // ── Generar PDF de evidencia ──────────────────────────────────────────────
  _origLog("\nGenerando PDF de evidencia...");
  const pdfPath = await generateEvidencePDF(capturedLogs);
  _origLog(`  PDF guardado en: ${pdfPath}`);
}

main().catch(async (e) => {
  console.error("\n ERROR:", e.message);
  _origLog("\nGenerando PDF de evidencia (con error)...");
  try {
    const pdfPath = await generateEvidencePDF(capturedLogs);
    _origLog(`  PDF guardado en: ${pdfPath}`);
  } catch (pdfErr) {
    _origLog("  No se pudo generar el PDF:", (pdfErr as Error).message);
  }
  process.exit(1);
});
