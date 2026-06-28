# 2026-06-27 Minecraft MC-17 D3 quality verdict live closure

Date: 2026-06-27

Status: live closure for dual threshold profiles (screenshot_copy_probe + trained_render).

Design: docs/ai/references/2026-06-27-minecraft-mc17-d3-quality-verdict-design.md

## Probe profile (reuse D2 store)

Store: .tmp/mc17-d2-live/store

Run: run_1782594531314_61141_0 (screenshot-copy MC-17 from D2 closure)

Observed via mc17_quality_baseline_report --verdict-mode probe:

- evidence_coverage=complete
- quality_verdict=pass under screenshot_copy_probe thresholds
- l1_mean=0, mse=0 (copy-probe baseline)

Honest boundary: pass means pipeline comparability on the fixed profile, not trained-splat usefulness.

## Trained-render profile (fresh evidence attempt)

Environment: ns-render / ns-train not on PATH; MC-9 normalized checkpoint is a 32-byte stub (step-000001.ckpt). Real trained splat render is unavailable locally.

Attempted MC-17 with explicit blocked render command under .tmp/mc17-d3-live/store.

Run: run_1782617683140_10406_0

Producer outcome: status=failed, verdict=failed, no metrics.

Derived D3 verdict on that run:

- evidence_coverage=partial (MC-12 spatial query resolved from store scan; MC-17 render failed)
- quality_verdict=blocked

### Copy-probe evidence judged under trained_render thresholds

On D2 copy-probe run run_1782594531314_61141_0, applying trained_render thresholds yields quality_verdict=partial because psnr is omitted when mse=0 while psnr_min=20.0 is required. This demonstrates the D3 invariant: evidence_coverage=complete does not imply quality_verdict=pass under the splat profile.

## Inspect acceptance

Against .tmp/mc17-d2-live/store / run_1782594531314_61141_0:

- MC-17 Quality Verdict: section present
- probe line includes quality_verdict=pass

## Unit-test coverage

Threshold policy is test-locked in src/run_read.rs (quality_baseline_verdict_* tests) and src/inspect.rs (render_run_text_renders_mc17_d2_quality_baseline_report).
