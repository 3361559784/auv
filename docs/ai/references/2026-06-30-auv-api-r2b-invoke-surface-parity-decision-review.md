# API-R2b: Invoke-Surface Parity Decision Review

**Date:** 2026-06-30  
**Status:** **owner accepted Package A (session-only freeze)** — final decision record.
Does not approve **API-R2b-impl**. P14 pause boundary unchanged.

**Prior work:** [API-R1](2026-06-30-auv-api-r1-invoke-operation-result-persistence-decision-review.md)
(decision review) → [API-R2](2026-06-30-auv-api-r2-invoke-operation-result-handoff.md)
(session invoke write-through, landed).

## One-line summary

API-R2 closed the `Invoke → GetOperation` gap **only on the session API path**.
Shared `auv-cli-invoke::recorded` (CLI, MCP, in-span callers) still does not
persist synthetic `operation-result` (or `operation-summary`) artifacts. This
review decides whether that divergence is an **intentional boundary** to freeze,
or whether **invoke-surface parity** should be planned — before any Rust changes.
**Owner answer: yes — intentional session boundary** (Package A).

## Owner freeze block

```text
synthetic operation-result write-through：session_service-only
synthetic operation-summary write-through：session_service-only（P11+R2b-A）
MCP/CLI catalog invoke：trace + inspect only（no join artifacts）
R2b-impl：frozen unless owner reopens with Package B + named consumer
```

### English expansion (for reviewers)

| Statement | Meaning | Evidence |
| --- | --- | --- |
| Operation-result session-only | Synthetic `operation-result` write-through lives in `session_service` only | [`handler.rs`](../../src/api/session_service/handler.rs) `finish_invoke_response`, [`operation_result_store.rs`](../../src/api/session_service/operation_result_store.rs) |
| Summary session-only | `operation-summary` persist remains `session_service`-only (P11 + R2b-A) | [`summary_store.rs`](../../src/api/session_service/summary_store.rs) |
| MCP/CLI trace-only | Catalog invoke through shared `invoke_recorded` leaves trace + inspect only | [`recorded.rs`](../../crates/auv-cli-invoke/src/recorded.rs), [`mcp.rs`](../../src/mcp.rs), [`main.rs`](../../src/main.rs) |
| R2b-impl frozen | No cross-frontend persist module unless owner signs Package B | This note |

## Slice classification

| Item | Value |
| --- | --- |
| This note (API-R2b) | **docs-only** |
| Follow-on code (API-R2b-impl, if approved) | **owner-approved feature** |
| Not | bug fix, test-only, narrow refactor |

## Problem / why now

API-R2 implemented owner package **D2-A** from API-R1: synthetic
`operation-result` write-through lives in `session_service::operation_result_store`
and is wired from `finish_invoke_response` only. The R2 handoff explicitly deferred
**API-R2b** (invoke-crate / MCP / CLI parity).

R1 flagged the main con of D2-A: **two invoke durability models** — session API
runs leave join artifacts; catalog invoke runs through shared `invoke_recorded` do
not. With R2 landed, that asymmetry is real and needs an explicit owner decision
before anyone extends persist logic to other frontends.

## Evidence: surfaces that differ after R2

### Durability matrix

| Surface | `operation-summary` persist | `operation-result` persist | Primary consumer |
| --- | --- | --- | --- |
| [`handler.rs` `finish_invoke_response`](../../src/api/session_service/handler.rs) L125–150 | **yes** (`record_invoke_summary`) | **yes** (`record_invoke_operation_result`) | `GetOperation` two-source join |
| [`invoke_recorded_with_session`](../../crates/auv-cli-invoke/src/recorded.rs) L33–76 | **no** | **no** | Returns `InvokeResult` only |
| [`invoke_recorded`](../../crates/auv-cli-invoke/src/recorded.rs) L17–22 | **no** | **no** | Same (default session) |
| [`mcp.rs` invoke tool](../../src/mcp.rs) L75–88 | **no** | **no** | JSON tool result; `run_inspect` for read-back |
| [`main.rs` `CliCommand::Invoke`](../../src/main.rs) L1351–1354 | **no** | **no** | Terminal output; `auv inspect` for read-back |
| [`invoke_recorded_in_span`](../../crates/auv-cli-invoke/src/recorded.rs) callers (`scroll_scan`, `app/infra`) | **no** | **no** | Parent run / span context |
| `run_recorded_operation` typed paths | varies by producer | **real typed** `OperationResult` | Inspect / vertical consumers |

### Flow (post-R2)

```text
Session API:
  Invoke → invoke_recorded_with_session → finish_invoke_response
         → operation-summary (P11) + operation-result (R2)
         → GetOperation join succeeds (happy path)

CLI / MCP:
  invoke → invoke_recorded → InvokeResult
         → trace + command artifacts only
         → run_read::read_operation_result → None
         → no GetOperation RPC on these surfaces today
```

### Additional divergence facts

| Fact | Location |
| --- | --- |
| P11 set the same session-only pattern for summary durability | [P11 handoff](2026-06-30-auv-api-p11-summary-durability-handoff.md) — write path is `session_service::summary_store` only |
| `auv-cli-invoke` has no `contract::OperationResult` dependency | [`crates/auv-cli-invoke/Cargo.toml`](../../crates/auv-cli-invoke/Cargo.toml) |
| `OperationResult`-sourced half intentionally not modeled in invoke crate | [`summary.rs` TODO L67–75](../../crates/auv-cli-invoke/src/summary.rs) |
| `InvokeCommandOutput.known_limits` still become span events only | [`recorded.rs` L176–183](../../crates/auv-cli-invoke/src/recorded.rs) — R2 D4-B unchanged |
| Synthetic honesty marker is session-namespaced | `auv.api.session.invoke_synthetic_operation_result` — [R2 handoff](2026-06-30-auv-api-r2-invoke-operation-result-handoff.md) |
| Read path unchanged; only store shape differs by frontend | [`run_read::read_operation_result`](../../src/run_read.rs) L1339–1352 |

### P11 precedent

API-P11 closed the **InvokeResult half** of the `GetOperation` join
(`operation-summary` artifact) on the **session handler path only**. It did not
extend `auv-cli-invoke::recorded`. API-R2 mirrored that pattern for the
**OperationResult half**. API-R2b is therefore the natural place to decide
**both halves together** for non-session invoke surfaces — not operation-result
alone in isolation.

## Options analysis

### Case for freeze (session-only) — reviewer recommendation

1. **`GetOperation` is a session API RPC.** CLI and MCP have no read-back
   equivalent today ([R1 anti-misread #5](2026-06-30-auv-api-r1-invoke-operation-result-persistence-decision-review.md)).
   Parity without a consumer is speculative scope.

2. **R1 owner package was D2-A.** R2 handoff non-goals explicitly list R2b.
   Extending persist without this review would violate pause discipline.

3. **P11 already established the boundary.** “GetOperation durability lives at
   the session API boundary,” not in `auv-cli-invoke`. R2 did not widen that;
   R2b would be the first cross-frontend durability change.

4. **Crate-graph cost of literal D2-B.** Moving persist into
   `auv-cli-invoke::recorded` requires `contract::OperationResult` in the invoke
   crate or duplicated wire types — both trigger
   [`CONTRIBUTING.local.md`](../../CONTRIBUTING.local.md) veto risks (duplicate
   contract, unclear owning boundary).

5. **Three quality tiers, not one gap.** Catalog invoke runs are
   observation/fixture class. Typed commands that need semantic
   `OperationResult` use `run_recorded_operation`. Session invoke uses
   **synthetic** skeleton with honesty marker. Conflating these tiers would
   mislead inspect consumers.

6. **No failing test or product path blocked.** Session API happy path is
   closed (R2). MCP/CLI workflows use trace + inspect text, not
   `GetOperation`.

**Reviewer recommendation:** **Package A — freeze session-only** unless owner
names a concrete consumer that requires MCP/CLI catalog invoke runs to be
`GetOperation`-ready (or `read_operation_result`-present) without going through
session `Invoke`.

### Case for parity (plan API-R2b-impl)

1. **Shared store workflows.** MCP and CLI can target the same `store_root` as a
   session server. Operators may pass `run_id` across surfaces and expect
   consistent artifact layout.

2. **Asymmetric store shape.** After session invoke, `read_operation_result`
   succeeds; after CLI/MCP catalog invoke, it returns `None` for the same command
   class — surprising when inspecting runs side by side.

3. **P11 never got a formal P11b decision.** Summary durability was also
   session-only without an explicit parity review. R2b is the right moment to
   decide both halves for non-session paths or freeze both intentionally.

4. **R1 documented the con.** “Two invoke durability models” was the main
   downside of D2-A; R2b is the named follow-up.

Parity is **reasonable** if a named consumer exists; it is **not required** for
session API correctness post-R2.

## Candidate API-R2b-impl slice (not approved here)

**If** owner accepts Package B below, the narrowest honest implementation:

| Step | Owner | Notes |
| --- | --- | --- |
| Extract shared write helpers | Main crate, e.g. `src/api/invoke_durability.rs` | Lift persist/build logic from `summary_store` + `operation_result_store`; keep marker policy explicit |
| Rewire session handler | `finish_invoke_response` | Call shared module; no behavior change; existing tests stay green |
| Wire top-level frontends only | `mcp.rs` invoke, `main.rs` `CliCommand::Invoke` | After `invoke_recorded` / `invoke_recorded_with_session` returns |
| **Pair both artifacts** | Same call sites | Summary + operation-result together — not operation-result alone |
| Exclude in-span callers | `invoke_recorded_in_span` (`scroll_scan`, `app/infra`) | Child spans are not standalone `GetOperation` targets |
| Marker policy | Owner decision | Reuse `auv.api.session.*` vs new `auv.invoke.synthetic_operation_result` |
| Tests | Hermetic | Shared module unit test + one MCP or CLI test that `read_operation_result` succeeds for `fixture.observe` — **not** full GetOperation (no MCP GetOperation surface) |
| Partial-success on MCP/CLI | Defer | No proto `InvokeResponse`; surface durability gaps via NOTICE or defer **API-R2c** |

### API-R2b-impl non-goals

- Persist inside `auv-cli-invoke::recorded` (literal D2-B — widens crate graph)
- `known_limits` plumbing from `InvokeCommandOutput` (**API-R2c** / D4-A)
- Operation-result-only backfill without summary parity
- P10 `StreamSessionEvents`, proto changes, inspect-server merge
- All `invoke_recorded_in_span` call sites
- Adding `GetOperation` to MCP or CLI
- Changing `GetOperation` join semantics or P12 wire identity
- Typed `OperationResult` for every catalog command (D2-C)

### API-R2b-impl validation floor (candidate)

```sh
cargo fmt --check
cargo check
cargo test session_service
cargo test read_operation_result
git diff --check
```

Plus narrow new tests for shared module and one frontend wiring path.

## Anti-misread rules (frozen)

1. **Synthetic `OperationResult` ≠ typed runtime record** — runs carrying
   `invoke_synthetic_operation_result` must not be treated as full semantic
   verification evidence in inspect or downstream automation.

2. **R2b parity ≠ GetOperation on MCP/CLI** — parity means **store artifact
   shape** alignment, not adding a read-back RPC to non-session frontends.

3. **P12 wire `operation_id` stays `command_id`** — internal domain label in
   JSON artifact may equal `command_id` for catalog invoke without changing wire
   rules.

4. **P11 partial-success policy applies** — persist failure after successful
   invoke must not fail invoke execution (non-idempotent blind-retry risk).

5. **P11 + R2 session-only frozen by owner Package A** — GetOperation durability
   is **session boundary** responsibility; MCP/CLI catalog invoke stays trace-only.

6. **Three tiers:** typed `run_recorded_operation` > session synthetic >
   catalog invoke trace-only. Do not collapse tiers.

7. **API-R2b review ≠ API-R2b-impl auto-start** — owner must name impl slice
   explicitly after signing Package B.

8. **This review does not unlock P10 or MCP merge** — see
   [P14](2026-06-30-auv-api-p14-api-line-closeout-pause-decision.md).

9. **R2b freeze ≠ R2c freeze** — limits plumbing decision remains separate
   ([R2c review](2026-06-30-auv-api-r2c-known-limits-plumbing-decision-review.md)).

10. **Signing R2b Package A does not sign R2c Package A** — independent owner
    decisions.

## Explicit non-goals (API-R2b review)

This note does **not** approve:

- Rust, proto, or transport code changes
- P10 `StreamSessionEvents`
- MCP / inspect-server unification
- D4-A `InvokeCommandOutput.known_limits` plumbing (**API-R2c**)
- Reopening controller / planner / lease / archived AX copilot lanes

## Owner decision packages

Answer **before** any API-R2b-impl work.

### Package A — Keep session-only (**accepted**)

**Accepted: 2026-06-30, Package A** — matches reviewer recommendation.

```text
R2b-A  Freeze: synthetic operation-result write-through remains session_service-only
R2b-B  Document P11+R2 as intentional "GetOperation durability at session boundary"
R2b-C  Defer shared-module extraction until a named consumer requires parity
R2b-D  Optional: P14 errata — note R2 closed session Invoke→GetOperation happy path
```

**When to choose:** Session API is the only GetOperation consumer; MCP/CLI
continue to use trace + inspect; no product path requires catalog invoke runs to
carry join artifacts.

### Package B — Approve API-R2b-impl — **not accepted**

```text
R2b-A  Accept parity for top-level CLI + MCP invoke (paired summary + operation-result)
R2b-B  Shared main-crate durability module; session_service becomes caller
R2b-C  Exclude invoke_recorded_in_span nested callers
R2b-D  Marker policy: <owner chooses session marker vs invoke-generic marker>
R2b-E  No auv-cli-invoke → contract dependency in v1
```

**When to choose:** A named workflow requires MCP/CLI catalog invoke runs on a
shared `store_root` to expose `read_operation_result` (and summary) without
session `Invoke`.

## Open questions — resolved (Package A)

| # | Question | Package A resolution |
| --- | --- | --- |
| 1 | P11 freeze together? | **Yes** — both `operation-summary` and `operation-result` frozen session-only |
| 2 | Package B marker policy | **N/A** — Package B not accepted |
| 3 | Package B partial-success surface | **N/A** — defer to R2c if R2b ever reopens |
| 4 | P14 staleness | **Resolved** — companion P14 errata in this slice |

## Reopen triggers

| Trigger | Unlocks | Does **not** auto-unlock |
| --- | --- | --- |
| Owner names **Package B** + concrete MCP/CLI catalog consumer on shared `store_root` | API-R2b-impl candidate only | R2c-impl, P10, MCP merge |
| Owner names **R2c-impl** | Command-limit plumbing only | R2b-impl |

## Relationship to R1 / R2 / R2c / P14 pause

- **R1** decided the write-through split; R2b closes the non-session parity question.
- **R2** landed session synthetic write-through only.
- **R2c** froze command `known_limits` plumbing — **independent** of this R2b freeze.
- **P14** pause boundary unchanged; companion errata records post-R2/R2b/R2c state.

## Validation (readers re-checking evidence)

```sh
rg -n "record_invoke_operation_result|record_invoke_summary" src crates
rg -n "invoke_recorded" src/mcp.rs src/main.rs
cargo test session_service
git diff --check
```

Expected: `record_invoke_*` appears only under `src/api/session_service/`; MCP and
CLI call `invoke_recorded` without follow-on persist.

## Related

- [API-R1 decision review](2026-06-30-auv-api-r1-invoke-operation-result-persistence-decision-review.md) (D2-A / R2b naming)
- [API-R2 handoff](2026-06-30-auv-api-r2-invoke-operation-result-handoff.md) (landed session write-through)
- [API-P11 summary durability](2026-06-30-auv-api-p11-summary-durability-handoff.md) (session-only summary precedent)
- [API-P14 pause decision](2026-06-30-auv-api-p14-api-line-closeout-pause-decision.md)
- [API-P4 server seam design](2026-06-30-auv-api-p4-session-proto-server-seam-design.md) (summary-source seam)
- `src/api/session_service/operation_result_store.rs`
- `src/api/session_service/summary_store.rs`
- `crates/auv-cli-invoke/src/recorded.rs`
