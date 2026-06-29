# API-P14: API Line Closeout / Pause Decision

**Date:** 2026-06-30  
**Status:** **final closeout + pause decision** — the approved session API unary lane
and P13 external client smoke are closed for their named scope. This note records
what landed from P1 through P13, which gaps are intentional deferrals, and which
owner-named triggers would reopen work. **No new implementation is approved by this
note.**

## One-line summary

The session API **unary** surface (`CreateSession` / `Invoke` / `GetOperation`) and
**P13 external client smoke** are landed and test-backed. **`StreamSessionEvents`**
remains unwired (P10 deferred). **Invoke → `OperationResult` persistence** remains a
**known gap** — not a P12 regression. Further API lane work requires an explicit
owner-named slice; pause does not imply P10 or operation-result wiring is "next."

## Owner freeze block

```text
unary 已有：CreateSession / Invoke / GetOperation
external smoke 已有：P13
stream 仍未启用：P10 defer
Invoke -> OperationResult 仍未打通：known gap
```

### English expansion (for reviewers)

| Statement | Meaning | Evidence |
| --- | --- | --- |
| Unary landed | All three unary RPCs wired through handler + loopback gRPC transport | [`handler.rs`](../../src/api/session_service/handler.rs), [`transport.rs`](../../src/api/session_service/transport.rs) |
| External smoke landed | Real `SessionServiceClient` over loopback TCP | [P13 handoff](2026-06-30-auv-api-p13-external-client-smoke-handoff.md), [`client_smoke.rs`](../../src/api/session_service/client_smoke.rs) |
| Stream not enabled | `StreamSessionEvents` returns `UNIMPLEMENTED` / `NotWired` | [`transport.rs` L206–212](../../src/api/session_service/transport.rs), [`handler.rs` L208–225](../../src/api/session_service/handler.rs) |
| Invoke → OperationResult gap | Fresh `Invoke` writes `operation-summary` only; `GetOperation` needs persisted `operation-result` | Handler NOTICE L11–15; P13 Journey B `FailedPrecondition` |

## Scope boundary

**In scope for this note:**

- Inventory of P1–P13 landings
- Frozen capability matrix and anti-misread rules
- Pause boundary before P10 / operation-result / MCP merge
- Reopen triggers for future owner-named slices

**Out of scope:**

- New Rust, proto, or transport code
- Implementing P10 `StreamSessionEvents`
- Wiring `OperationResult` on `Invoke`
- MCP / inspect-server unification
- Controller / planner / lease / archived AX copilot lanes

This is a **boundary record**, not a proposal to continue implementation.

## Closed phases (P1–P13)

"Closed" means the slice reached its intended endpoint for the named scope. It does
not mean every proto RPC is fully featured.

| Phase | Closure type | Pointer | What "closed" means |
| --- | --- | --- | --- |
| **P1** | Boundary review | [`2026-06-30-auv-api-p1-session-proto-boundary-review.md`](2026-06-30-auv-api-p1-session-proto-boundary-review.md) | Proto surface vocabulary and experimental stability language frozen |
| **P2** | Proto + crate | `proto/auv/api/v1/session.proto`, `crates/auv-api-proto` | Generated types available to server and clients |
| **P3** | Mapper boundary (docs) | [`2026-06-30-auv-api-p3-session-proto-mapper-boundary-handoff.md`](2026-06-30-auv-api-p3-session-proto-mapper-boundary-handoff.md) | Two-source `GetOperation` join documented; open decisions catalogued |
| **P4** | Server seam design + modules | [`2026-06-30-auv-api-p4-session-proto-server-seam-design.md`](2026-06-30-auv-api-p4-session-proto-server-seam-design.md), `src/api/session_service/` | Dedicated session API boundary; not MCP/inspect glue |
| **P5** | Session-aware invoke | `handler.rs` `invoke_recorded_with_session` | Runs stamp explicit `session_id` on recorded runs |
| **P6** | Summary cache | `OperationSummaryCache` in handler | Process-local runtime summary for same-handler `GetOperation` |
| **P7** | Two-source join | `summary.rs` | Explicit join policy for `GetOperation` |
| **P8** | Handler skeleton | `handler.rs` | Transport-agnostic unary RPC wiring |
| **P9** | Loopback gRPC | `transport.rs` | `CreateSession`, `Invoke`, `GetOperation` over tonic |
| **P11** | Summary durability | [`2026-06-30-auv-api-p11-summary-durability-handoff.md`](2026-06-30-auv-api-p11-summary-durability-handoff.md) | `operation-summary` artifact persisted on `Invoke` |
| **P12** | Identity / role closeout | [`2026-06-30-auv-api-p12-identity-role-semantics-closeout.md`](2026-06-30-auv-api-p12-identity-role-semantics-closeout.md) | Wire `operation_id` = `command_id`; `ArtifactRef.role` from catalog |
| **P13** | External client smoke | [`2026-06-30-auv-api-p13-external-client-smoke-handoff.md`](2026-06-30-auv-api-p13-external-client-smoke-handoff.md) | Three gRPC smoke journeys including honest GetOperation precondition gap |

## Frozen capability matrix

| Capability | Status | Notes |
| --- | --- | --- |
| `CreateSession` | **landed** | Lightweight registry; no `SessionRuntime` materialization |
| `Invoke` (blocking unary) | **landed** | Records run + `operation-summary`; returns `InvokeResponse` |
| `GetOperation` (with persisted skeleton) | **landed** | Two-source join when `operation-result` artifact exists |
| `GetOperation` after fresh `Invoke` only | **known gap** | `FAILED_PRECONDITION` / `PersistedOperationRequired` — by design until operation-result slice |
| External client smoke (P13) | **landed** | `cargo test session_api_smoke` |
| `StreamSessionEvents` (P10) | **deferred** | Transport `UNIMPLEMENTED`; handler `NotWired` |
| `json_payload` envelope (P3 OD5) | **deferred** | Provisional decoder only; owner-named envelope slice required |

## Unary path invariant (frozen)

```text
CreateSession → register session_id
Invoke(session, command_id) → recorded run + operation-summary artifact + InvokeResponse
GetOperation(run_id) → join operation-result + summary when skeleton exists
```

`GetOperation` after `Invoke` alone is **not** part of this invariant until an
owner-named **operation-result on Invoke** slice lands.

## Anti-misread rules

These rules are part of the pause boundary.

### 1. Fresh Invoke → GetOperation failure is expected

`Invoke` does not persist `OperationResult`. `GetOperation` requires it. P13
Journey B documents this from the external client view. Treating it as a P12
identity regression is **wrong**.

### 2. Stream UNIMPLEMENTED is not a transport regression

`StreamSessionEvents` was never wired. P10 is the named future slice. Absence of
streaming is a **documented deferral**, not missing polish on unary RPCs.

### 3. Two different `SessionEvent` types

`src/session.rs::SessionEvent` is observation/action-resource oriented.
`auv.api.session.v1.SessionEvent` is invoke/run/artifact oriented. Mapping one to
the other is a category error (see API-P4 §D).

### 4. P13 smoke ≠ production external API

P13 uses in-process loopback TCP and hermetic fixtures. It does not certify
subprocess `auv session serve` CI, remote access, TLS, or gRPC reflection.

### 5. Pause does not unlock adjacent lanes

P14 closeout does **not** approve MCP/proto server unification, inspect-server
merger, controller, planner, or action lease work.

## Anti-misread rule (main point)

> **API-P14 closeout means "the approved unary session API lane + P13 smoke are
> done for their named scope."** It does **not** mean "P10 stream or
> operation-result wiring is the obvious next implementation."

### Forbidden misreads

- "P13 smoke is green, so Invoke → GetOperation must work without fixtures."
- "Stream is in the proto, so unary closeout should have included P10."
- "P12 fixed identity, so GetOperation precondition failures are bugs."
- "Session API pause means we should merge execute API into inspect_server or MCP."

## Explicit non-goals (P14)

API-P14 does **not** approve:

- implementing P10 `StreamSessionEvents` in this slice
- persisting `OperationResult` on session `Invoke`
- expanding `session.proto` or adding gRPC reflection
- subprocess / grpcurl CI gates for session API
- MCP / inspect-server route unification
- `json_payload` envelope standardization (P3 OD5)
- reopening controller / planner / lease / archived `candidate-action` lanes

## Reopen triggers

A paused lane does not reopen because it "feels next." It reopens only when the
owner names the trigger **and** the exact slice.

| Trigger | Unlocks (candidate only) | Does **not** auto-unlock |
| --- | --- | --- |
| Owner names **P10** | `StreamSessionEvents` v0 (handler-emitted hub) | RunUpdate projector, `invoke_started`, proto expansion |
| Owner names **operation-result on Invoke** | Fresh-store Invoke → GetOperation without fixture | Stream, MCP merge |
| Owner names **P3 OD5 envelope** | Versioned `json_payload` decoder | Stream, operation-result |
| Owner names **P13b** | Subprocess `auv session serve` smoke | Unary semantics change |
| Owner names **P10b** | RunUpdate / BroadcastRunRecorder projection | Unary changes |

**Trigger met ≠ implement.** A reopened lane still needs a named slice and fresh
scope review against `CONTRIBUTING.local.md`.

## Validation (re-check state)

Readers verifying this pause record against the repo:

```sh
cargo test session_service
cargo test session_api_smoke
git diff --check
```

Expected: `session_service` includes handler, mapper, summary, transport, and
`client_smoke` tests; `session_api_smoke` runs three external-client journeys.

## Related

- API-P1 boundary review:
  [`2026-06-30-auv-api-p1-session-proto-boundary-review.md`](2026-06-30-auv-api-p1-session-proto-boundary-review.md)
- API-P3 mapper boundary:
  [`2026-06-30-auv-api-p3-session-proto-mapper-boundary-handoff.md`](2026-06-30-auv-api-p3-session-proto-mapper-boundary-handoff.md)
- API-P4 server seam:
  [`2026-06-30-auv-api-p4-session-proto-server-seam-design.md`](2026-06-30-auv-api-p4-session-proto-server-seam-design.md)
- API-P11 summary durability:
  [`2026-06-30-auv-api-p11-summary-durability-handoff.md`](2026-06-30-auv-api-p11-summary-durability-handoff.md)
- API-P12 identity closeout:
  [`2026-06-30-auv-api-p12-identity-role-semantics-closeout.md`](2026-06-30-auv-api-p12-identity-role-semantics-closeout.md)
- API-P13 external smoke:
  [`2026-06-30-auv-api-p13-external-client-smoke-handoff.md`](2026-06-30-auv-api-p13-external-client-smoke-handoff.md)
- MC-20 pause template (pattern reference):
  [`2026-06-30-minecraft-mc20-final-closeout-pause-decision.md`](2026-06-30-minecraft-mc20-final-closeout-pause-decision.md)
- Proto: `proto/auv/api/v1/session.proto`
- Implementation: `src/api/session_service/`
