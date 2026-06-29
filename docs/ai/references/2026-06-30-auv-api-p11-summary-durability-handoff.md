# API-P11: Summary Durability Handoff

**Date:** 2026-06-30  
**Status:** Implemented  
**Slice:** Owner-approved feature — durable `InvokeResult`-sourced `GetOperation` half

## Problem resolved

API-P6 kept the runtime summary (`output_summary`, `signals`, `failure_message`) only in
an in-memory `OperationSummaryCache`. After process restart, `GetOperation` joined a
persisted `OperationResult` with no runtime source and surfaced
`auv.api.session.runtime_summary_unavailable`.

API-P11 closes that gap for the **InvokeResult-sourced half** only (API-P3 open decision 1).

## Design

| Piece | Owner | Notes |
|-------|-------|-------|
| Wire record | `auv_cli_invoke::OperationSummaryRecord` | Serde mirror of `OperationSummary` |
| Version constant | `contract::OPERATION_SUMMARY_API_VERSION` | `auv.operation_summary.v1alpha1` |
| Artifact role | `contract::OPERATION_SUMMARY_ARTIFACT_ROLE` | `operation-summary` |
| Write path | `session_service::summary_store::persist_operation_summary` | Write-through on `Invoke` |
| Read path | `run_read::read_operation_summary` | First JSON `operation-summary` artifact |
| Join | `session_service::summary::load_joined_operation_summary` | Cache override, else store |

Artifact JSON shape (illustrative):

```json
{
  "api_version": "auv.operation_summary.v1alpha1",
  "run_id": "<run_id>",
  "status": "completed",
  "output_summary": "...",
  "signals": { "k": "v" },
  "failure_message": null
}
```

## Behavior

1. **`Invoke`**: capture summary → RAM cache (API-P6) **and** append `operation-summary.json`
   on the recorded run (span = `InvokeResult.producer_span_id`).
2. **`GetOperation`**: join persisted `OperationResult` with runtime from cache when present;
   on cache miss, load from store artifact.
3. **Restart**: new `SessionApiHandler` over the same `store_root` returns full runtime fields
   without repopulating the cache (see `get_operation_survives_handler_restart_via_persisted_summary_artifact`).

## Duplicate artifacts

Invoke retries may append multiple `operation-summary` artifacts. The read path takes the
**first** match, mirroring `read_operation_result`.

## Deferred (not this slice)

- `OperationResult` persistence on `Invoke` (`Runtime::record_operation` wiring)
- Session registry durability across restart
- Proto / mapper field shape changes
- `StreamSessionEvents` / event projector
- Replacing the in-memory cache (hybrid write-through only)

## Verification

```sh
cargo fmt --check
cargo check
cargo test session_service
cargo test operation_summary
cargo test read_operation_summary
```

## References

- `docs/ai/references/2026-06-30-auv-api-p4-session-proto-server-seam-design.md` (summary-source seam)
- `src/api/session_service/summary_store.rs`
- `src/run_read.rs` (`read_operation_summary`)
- `crates/auv-cli-invoke/src/summary.rs` (`OperationSummaryRecord`)
