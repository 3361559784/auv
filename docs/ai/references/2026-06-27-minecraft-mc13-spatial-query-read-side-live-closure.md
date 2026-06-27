# MC-13 spatial query read-side live closure

Date: 2026-06-27

## Summary

This note records read-side / inspect consumption of MC-12 spatial query artifacts
from an existing MC-12 live run. MC-13 adds no new CLI and does not rerun spatial
query; it only closes `run_read`, `auv inspect` text, and inspect-viewer summary
cards for:

- `minecraft-3dgs-training-result-query`
- `minecraft-3dgs-training-result-query-inspect`

This is **read-side evidence closure** only. It does **not** claim action-path
readiness, Gaussian-native inference, or render preview.

## Reused MC-12 live run

From `docs/ai/references/2026-06-27-minecraft-mc12-spatial-query-live-closure.md`:

- MC-12 spatial query run (visible / reference-only gate):
  `run_1782543398186_14786_0`
- Query manifest artifact:
  `artifacts/artifact_0001_minecraft-3dgs-training-result-query.json`
- Query inspect artifact:
  `artifacts/artifact_0002_minecraft-3dgs-training-result-query-inspect.json`

## Inspect text gate

```sh
cargo run --quiet -- inspect run_1782543398186_14786_0
```

Observed MC-13 section:

```text
MC-12 Training Result Spatial Query:
- manifest_artifact=artifact_0001 ... target_block=511,73,728 ...
  selected_backend=projection_reference status=answered visibility=visible
  paired_report_artifact=artifact_0002 ...
  paired_report schema=1 provider_status=blocked reference_status=answered
  comparison_verdict=reference_only visibility=visible scene_packet_frame_count=6 ...
```

Gate checks:

- `MC-12 Training Result Spatial Query:` section present
- business-key pairing renders `paired_report_artifact=artifact_0002`
- query identity fields visible: `target_block`, `selected_backend`, `visibility`
- dual-backend inspect fields visible: `provider_status`, `reference_status`,
  `scene_packet_frame_count`

## Viewer smoke (manual)

1. Start inspect server for a run containing MC-12 query artifacts (for example
   `run_1782543398186_14786_0`).
2. Open each query manifest / inspect artifact.
3. Confirm summary card appears above raw JSON with `status`, `selected_backend`,
   `target_block`, and inspect-only fields (`provider_status`, `reference_status`).
4. Confirm raw JSON text and download entry remain available.

## Verdict

MC-13 is **live-closed** for read-side consumption of MC-12 query manifest /
inspect pairs on the MC-12 visible reference-only gate run. Pairing uses business
lineage + query identity keys only (no artifact-path shortcut).

## Related references

- MC-13 design:
  `docs/ai/references/2026-06-27-minecraft-mc13-spatial-query-read-side-inspect-consumer-design.md`
- MC-12 live closure:
  `docs/ai/references/2026-06-27-minecraft-mc12-spatial-query-live-closure.md`
