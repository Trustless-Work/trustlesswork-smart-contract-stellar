## XLM Flow Testing — Evidence Report

**Network:** Stellar Testnet
**Contract ID (main flow):** CB5FLCYCMXNRYLUB6XPPK6QWPPGWLTCTNFJNKISLP2GD5XZHVVK4LB5N
**Contract ID (dispute flow):** CBTQDCRSUSUEUK2GPAWDY3F5CT6WBNT6Z5JW73E7ICYCEL55767WWGJX
**XLM SAC address used:** CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC

### Transaction Hashes

| Step | Function | TX Hash | Stellar Expert Link |
|---|---|---|---|
| 3 | Deploy | `f01dbb3e4bacaafb8a804bc27187543e78a765a54ba0b7a3f83ddd49ecfa0257` | [link](https://stellar.expert/explorer/testnet/tx/f01dbb3e4bacaafb8a804bc27187543e78a765a54ba0b7a3f83ddd49ecfa0257) |
| 4 | `initialize_escrow` | `a75dc868f3fab821264318549ba73d5daac76409d79c22997c871c68f7b458b1` | [link](https://stellar.expert/explorer/testnet/tx/a75dc868f3fab821264318549ba73d5daac76409d79c22997c871c68f7b458b1) |
| 5 | `fund_escrow` | `47b9c1e55672e98d2344f232f223e7c54a3d8400f5e67946472f7f80e00cb648` | [link](https://stellar.expert/explorer/testnet/tx/47b9c1e55672e98d2344f232f223e7c54a3d8400f5e67946472f7f80e00cb648) |
| 6a | `change_milestone_status` (0) | `235eeb02a459b1bf1beb4da4098a5e4fb06028affc091cc17835f6c75568c609` | [link](https://stellar.expert/explorer/testnet/tx/235eeb02a459b1bf1beb4da4098a5e4fb06028affc091cc17835f6c75568c609) |
| 6b | `change_milestone_status` (1) | `51cde352745b871a37c4fa3f80a91bb80c0b5a1adee58fc848daee734e2dde46` | [link](https://stellar.expert/explorer/testnet/tx/51cde352745b871a37c4fa3f80a91bb80c0b5a1adee58fc848daee734e2dde46) |
| 7a | `approve_milestone` (0) | `8646493e481ca1cf55dc22f607d2a1b71dfe261321795fdfb30a2d32f3061975` | [link](https://stellar.expert/explorer/testnet/tx/8646493e481ca1cf55dc22f607d2a1b71dfe261321795fdfb30a2d32f3061975) |
| 7b | `approve_milestone` (1) | `38e0615a86f82e3168ea02e950e152efba2b8fa9328bf52a0aaf0afb50bd02c6` | [link](https://stellar.expert/explorer/testnet/tx/38e0615a86f82e3168ea02e950e152efba2b8fa9328bf52a0aaf0afb50bd02c6) |
| 8 | `release_funds` | `ef8563c036bd392ea27c285e4368da6edc471a52fd90cff062f933af447bb604` | [link](https://stellar.expert/explorer/testnet/tx/ef8563c036bd392ea27c285e4368da6edc471a52fd90cff062f933af447bb604) |
| 9 (opt) | `dispute_escrow` | `4e162ace378ee2a3de0afbca04218358c53463b14c86fba251f724373c493b42` | [link](https://stellar.expert/explorer/testnet/tx/4e162ace378ee2a3de0afbca04218358c53463b14c86fba251f724373c493b42) |
| 9 (opt) | `resolve_dispute` | `2545b57973a8d32a3d6b777bf52541a0b412e3ead37f2a2bd99099566bd87052` | [link](https://stellar.expert/explorer/testnet/tx/2545b57973a8d32a3d6b777bf52541a0b412e3ead37f2a2bd99099566bd87052) |

### Fee Verification (release_funds)

| Recipient | Expected (stroops) | Received (stroops) |
|---|---|---|
| Alice (receiver) | `20000000 - 60000 - 600000 = 19340000` | `19340000` |
| Trustless Work (0.30%) | `60000` | `60000` |
| Bob / platform (3%) | `600000` | `600000` |

### Fee Verification (resolve_dispute)

| Recipient | Distribution (stroops) | After fees (stroops) |
|---|---|---|
| Alice | `12000000` | `11604000` |
| Bob | `8000000` | `7736000` |
| Trustless Work (0.30%) | — | `60000` |
| Platform (3%) | — | `600000` |
| **Total** | **20000000** | **20000000** |

### Observations

1. **XLM nativo funciona correctamente como trustline.** El contrato escrow completó todo el ciclo de vida (inicializar, fondear, milestones, aprobaciones, release y disputa) sin errores relacionados al activo.

2. **Diferencias encontradas entre el issue y el contrato real:**
   - El campo `receiver_memo` es tipo `u32` en el contrato, no `string` como sugiere el issue. Pasar `"0"` (string) causa un error de parseo; debe ser `0` (número).
   - El campo `platform_address` en el struct `Roles` del issue se llama `platform` en el contrato real.
   - Los valores string en argumentos individuales del CLI (como `--new_status` y `--new_evidence`) requieren comillas JSON explícitas: `'"Completed"'` en lugar de `"Completed"`.

3. **Intermitencia del RPC de Testnet.** Se encontraron múltiples errores `503` del nodo RPC (`soroban-testnet.stellar.org`) durante la sesión. No están relacionados con XLM — son problemas de disponibilidad del servicio. Los comandos se resolvieron reintentando.

4. **Friendbot no disponible.** El servicio Friendbot para fondear cuentas de prueba devolvió `503`. Las cuentas se fondearon manualmente desde una wallet externa.

5. **XLM no requiere trustline previa.** A diferencia de USDC u otros activos emitidos, no fue necesario ejecutar `changeTrust` en ninguna cuenta antes de recibir XLM. Esto confirma el comportamiento esperado descrito en el issue.

6. **Distribución de fees en disputa.** En `resolve_dispute`, los fees (0.30% Trustless Work + 3% plataforma) se descuentan proporcionalmente de cada distribución antes de transferir, no del monto total. Esto es consistente con el comportamiento del contrato.
