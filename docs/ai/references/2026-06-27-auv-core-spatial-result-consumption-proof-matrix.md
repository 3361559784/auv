# 2026-06-27 AUV core spatial result consumption proof matrix

Date: 2026-06-27

Status: design-only graduation matrix. This note defines what evidence each
Core-A candidate contract still needs before any code extraction can be taken
seriously.

## Why this note exists

Core-A D2 already classified the Minecraft MC-10 through MC-17 surface into:

- keep app-specific
- extract helper only
- candidate core contract
- explicitly deferred

That still leaves one risk: people may read “candidate core contract” as
“basically ready, go extract it”.

This D4 note closes that gap. It says, for each candidate contract:

- what positive evidence is still required
- what counter-evidence or falsifier must be checked
- what the smallest acceptable extraction shape would be
- what would still disqualify extraction

## Non-goals

This note does **not**:

- extract code
- rename current Minecraft symbols
- introduce a new core crate
- approve helper extraction automatically
- broaden into generic provider traits or viewer unification

## Graduation rules that apply to every candidate

Before any candidate below can graduate into shared AUV code, all of these
must hold:

1. **Second-vertical evidence exists**
   - not another Minecraft slice
   - not a synthetic doc-only example
   - not just “this shape feels reusable”
2. **One positive and one negative path are both evidenced**
   - success-only evidence is not enough
   - the contract must preserve its honest failure split under pressure
3. **The extracted name is no longer donor-shaped**
   - no `minecraft`, `checkpoint`, `block`, `nerfstudio`, or equivalent donor
     vocabulary in the shared type
4. **The shared boundary is smaller than the donor implementation**
   - extract the contract or helper, not the whole vertical module
5. **An owning layer is explicit**
   - runtime contract, domain helper, or read-side helper must be named in
     advance

If any one of these fails, the candidate stays local.

## Candidate proof matrix

| Candidate contract | Current donor symbols | Positive evidence still required | Counter-evidence / falsifier still required | Smallest acceptable extraction shape | Current blockers | Current verdict |
| --- | --- | --- | --- | --- | --- | --- |
| Stage status triad | `TrainingResultSemanticStatus`, `HoldoutPreviewStatus`, `HoldoutRenderQualityStatus` | One non-Minecraft vertical must need the same `ready / blocked / failed` split across at least two persisted stages, with lineage-carrying artifacts. | Show a real case where one vertical needs a fourth state or where `blocked` and `failed` collapse without loss. If that happens, do not extract the triad yet. | Shared enum or label helper only; no shared manifest struct. | osu WQ1 adds semantic + witness + quality stages — see Core-A2 review³; Core-A3 helper landed⁴; default **defer** Core-B extraction. | `candidate, helper-only admissible`³ |
| Query status triad | `TrainingResultSpatialQueryStatus` | One non-Minecraft vertical must expose target-conditioned answers where `answered` must stay distinct from readiness or semantic success. | Show a real case where query answers are naturally boolean or where `answered` adds no information. If true, this stays app-local. | Shared query-status enum only. | Main matrix verdict unchanged pending owner acceptance of graduation review; osu adds probe-local recurrence only — see graduation review (admissibility-only, defer extraction). | `candidate, not admissible yet`¹ |
| Provider comparison verdict | `TrainingResultSpatialQueryComparisonVerdict` | Another vertical must actually run dual-backend answer compare and need the same `match / divergent / provider_only / reference_only / not_comparable` split in persisted evidence. | Show a real compare seam where ordering, partiality, or uncertainty needs extra states beyond the five-label set. | Shared compare-verdict enum or helper. | Only Minecraft provider/reference compare exists. | `candidate, not admissible yet` |
| Action readiness view | `TrainingResultSpatialQueryActionEligibility`, `TrainingResultSpatialQueryActionReadiness`, `derive_action_readiness`, `derive_minecraft_training_result_spatial_query_action_readiness` | Another vertical must need a **derived** action-facing view over persisted answer artifacts, with at least one `click_ready`-like path and one honest non-actionable path. | Show a real case where action-facing consumption must mutate producer truth or where dispatch and readiness cannot be kept separate. | Shared derived read-model contract only; no runtime dispatch wiring. | Main matrix verdict unchanged pending owner acceptance of graduation review; osu adds derived-shape recurrence only — capture-space, not dispatch-safe — see graduation review. | `candidate, not admissible yet`¹ |
| Quality measurement verdict | `HoldoutRenderQualityVerdict` | Another vertical must measure witness-bound quality evidence and need the same split between `measured_only`, `metric_partial`, `blocked`, and `failed`. | Show a real measurement seam where thresholds are inseparable from measurement evidence, or where partial measurement is meaningless. | Shared evidence-verdict enum only. | osu WQ1 probe-local recurrence strengthened³; `metric_partial` semantics differ from MC-17 — **not** helper-only admissible. | `candidate, not admissible yet` |
| Persisted backend label discipline | `HoldoutRenderQualityBackend`, `TrainingResultSpatialQueryBackend` | Another vertical must persist backend provenance and independently hit the same rule: stable backend labels belong in artifacts, raw runtime command text does not. | Show a real vertical where backend provenance cannot be represented by stable labels alone. | Shared label-discipline rule or tiny backend-label trait bound, if ever needed. | osu hits query-backend discipline only; render/quality backend still MC-only³. | `candidate, not admissible yet` |


## osu! second-vertical probe (2026-06-27)

Companion evidence:
`docs/ai/references/2026-06-27-auv-second-vertical-consumption-probe-osu-evidence.md`

This probe closes **MC-14-analog derived consumption** on osu! only (query→
readiness shape recurrence, not full MC-14 window-readiness parity). Full-chain
update (PR #54 wired live action + WQ1): see Core-A2 reviews³. WQ1 witness/quality
is probe-local recurrence (footnote²); live admission is Core-C1 donor evidence,
not matrix graduation.

| Candidate contract | Verdict after osu full chain | Notes |
| --- | --- | --- |
| Query status triad | **satisfied as second-vertical probe-local recurrence** | `answered` distinct from semantic `ready`; single backend only — **not extraction-pressure evidence**. |
| Action readiness view | **satisfied as second-vertical probe-local recurrence** | Derived triad + wired live admission (PR #54); **capture-space consumability** at readiness layer. |
| Stage status triad | **satisfied as second-vertical probe-local recurrence** | Semantic + witness + quality stages (`ready/blocked/failed`); main matrix helper-only admissible³. |
| Provider comparison verdict | **not satisfied** | Dual-backend compare intentionally deferred. |
| Quality measurement verdict | **satisfied as probe-local recurrence (OSU-WQ1)** | Four-label verdict chain; `metric_partial` semantics differ from MC-17 — main matrix **not** admissible³. |
| Persisted backend label discipline | **satisfied (query backend layer)** | `query_backend=playfield_projection_reference` persisted; render/quality backend enum still MC-only. |

Rows marked probe-local remain **non-admissible for Core-B extraction** unless
footnote³ helper-only admissible language applies (stage triad only on main matrix).
Graduation reviews:
`docs/ai/references/2026-06-27-auv-core-a-query-readiness-graduation-review.md` (rows 66/68)
and Core-A2³ (rows 65/69/70). **Admissible does not mean recommended now**;
default remains **defer**.

¹ See graduation review
`docs/ai/references/2026-06-27-auv-core-a-query-readiness-graduation-review.md`:
may record **helper-only admissible (review language only)** for these two rows
after owner acceptance — **not** an extraction recommendation; default **defer**.

² OSU-WQ1 probe-local evidence (quality row main matrix unchanged):
`docs/ai/references/2026-06-28-osu-wq1-witness-quality-evidence-design.md` and
`docs/ai/references/2026-06-27-auv-second-vertical-consumption-probe-osu-evidence.md`
(WQ1 section). Default remains **defer** extraction.

³ Core-A2 second-vertical graduation review (2026-06-28, `main` @ `91577c5`):
[`2026-06-28-auv-core-a2-stage-quality-graduation-review.md`](2026-06-28-auv-core-a2-stage-quality-graduation-review.md)
(stage triad helper-only admissible; quality + backend rows unchanged on main
matrix) and
[`2026-06-28-auv-core-a2-full-chain-falsifier-review.md`](2026-06-28-auv-core-a2-full-chain-falsifier-review.md)
(full-chain falsifier + Core-C1 re-review). **Admissible does not mean
recommended now**; default **defer** extraction.


⁴ Core-A3 helper extraction (2026-06-29, `feat/core-a3-stage-status-triad-helper`):
[`2026-06-29-auv-core-a3-stage-status-triad-helper-design.md`](2026-06-29-auv-core-a3-stage-status-triad-helper-design.md)
— `auv-stage-status::StageStatus` wired via donor type aliases; row 65 verdict
unchanged; **not** Core-B graduation.

## What does not need a second vertical

Not every reuse decision needs a full graduation process.

The following may extract earlier **as helpers only**, if repetition appears and
the extracted shape stays narrow:

- artifact JSON read helpers
- MIME / JSON gating helpers
- business-key unique-match helper
- narrow RGB metric math helper

These are still not automatic. The test is simpler:

- duplicated in more than one owned module
- extraction removes repetition without inventing a new project concept
- no donor-specific vocabulary leaks into the helper name

## Disqualifiers

Any of the following should stop a proposed graduation immediately:

- the shared type name is still Minecraft-shaped
- the extraction proposal moves whole manifests instead of the contract seam
- the proposal mixes runtime, provider, read-side, and viewer concerns together
- the proposal claims “likely reusable” without a second real consumer
- the proposal uses D2 candidate status as if it were extraction approval
- the proposal hides unresolved donor quirks behind a generic name

## Concrete next-step filter

If someone proposes Core-B extraction after this note, the first question is
not “is it elegant?”.

The first question is:

```text
Which exact row in the proof matrix is now satisfied by new evidence?
```

If that cannot be answered concretely, the extraction is premature.

## Relationship to Core-A D1–D3

- D1 froze the stage pattern:
  `docs/ai/references/2026-06-27-auv-core-spatial-result-consumption-pattern.md`
- D2 classified actual modules and donor symbols:
  `docs/ai/references/2026-06-27-auv-core-spatial-result-consumption-admission-table.md`
- D3 added the minimal vocabulary to:
  `docs/TERMS_AND_CONCEPTS.md`

D4 does not add more vocabulary. It adds graduation gates.
