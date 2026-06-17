# 2026-06-16 Minecraft live MC-2/MC-3/MC-4 closure plan

> AUV remains a skill substrate. The Minecraft sidecar is read-only truth and
> verifier. All actions go through the AUV driver. KG1/KG2/KG3 from the P0 doc
> remain in force.

Date: 2026-06-16

## Scope classification

`docs-only`

Why this classification is correct:

- This note sequences already-defined slices (MC-2 / MC-3 / MC-4) toward their
  existing live-acceptance gates. It adds no new contract, core surface, CLI
  command, or Minecraft noun.
- It exists so the next implementation slice starts from a stable, named
  boundary instead of re-deriving the lane.
- It is not approval to widen scope. It is the opposite: it pins the next effort
  to live closure and explicitly **parks** the perception fork.

## Owner decision being recorded

Direct the next effort at closing **live MC-2 / MC-3 / MC-4 end-to-end**, per the
`2026-06-15-minecraft-series-handoff.md` "recommended next order". The MC-6 / MC-7
perception fork (2.5D-measurement-first vs 3DGS) is **deliberately deferred** — see
"Deferred decision" below — and must NOT be started inside these slices.

## Current boundary (from the 06-15 handoff, restated)

Already true (crate-local / offline / read-side only):

- MC-2 projection + overlay logic — crate-local, offline geometry artifact.
- MC-3 world-diff verdict logic — offline.
- MC-4 mismatch-refusal logic — crate-local.
- MC-1 durable telemetry sample imported as a run artifact
  (`run_1781537928075_75501_0` / `artifact_0001`, local-only / gitignored).
- MC-3 / MC-4 read-side execution-evidence closure in core
  (`CandidateActionExecutionClosureState`: `evidence_closed` / `semantic_open` /
  `blocked_by_readiness`).

Not yet true (the live gap this plan closes):

- No live screenshot ↔ `SpatialFrame` binding on a running client.
- No live driver-on-client real-input proof.
- No real-client MC-4 refusal sample matrix.

So P0's one-line thesis — same-instant frame + projection + real input +
negative-case refusal — is proven only crate-local / offline. This plan makes it
**live**.

## Live-closure slices (one risk-coherent slice each; finish, report, stop)

### Slice 1 — MC-2 live screenshot/projection evidence bridge

Bind a real AUV window capture of the running Fabric client to a `SpatialFrame`
at the same instant (record `capture_skew_ms`), project a known `WorldTarget`
block `{x,y,z}`, and persist the overlay-on-real-frame as durable evidence
through the existing artifact seam — making MC-2 projection evidence first-class
without a new result family.

Acceptance gate:

- A real captured frame with the overlay (projected box + crosshair + raycast
  marker) visibly on the correct block.
- `capture_skew_ms` recorded; the reject path is exercised when skew is over
  threshold.
- Evidence persisted through the existing run/store seam and visible read-side.
  No core schema change, no MC nouns to core, no Mineflayer.

Most relevant files: `crates/auv-game-minecraft/src/{projection,artifact,overlay,input_target}.rs`;
only if needed for persistence/read-side visibility:
`src/{candidate_action_decision,run_read,inspect}.rs`.

### Slice 2 — MC-3 live real input + world-diff verify

Fixed local world, fixed marked target (e.g. a red-wool block at a known
`{x,y,z}`). Flow: `WorldTarget` → `ProjectedScreenTarget` → ActionResolver → the
**AUV driver** delivers real aim/click/hold into the MC window → query the
sidecar for the world diff (block → `air`, or inventory +1) → `VerificationResult`,
recorded with a run id. Reuse the seam and `auv-driver` unchanged.

Acceptance gate:

- The target block actually changes on the live client.
- The run records a passing `VerificationResult` with run id + world diff.
- KG2 held: zero Mineflayer / MCP action; every input is real driver delivery.

### Slice 3 — MC-4 live refusal matrix (KG3 — the real acceptance)

Trigger each mismatch class on the live client and prove AUV refuses with a
structured reason + `SpatialFrame` evidence through the existing refusal /
`VerificationResult` seam, feeding the already-closed read-side closure path (no
new schema):

```text
target window not Minecraft            → refuse
screenshot is menu / black / loading   → refuse
capture_skew_ms over threshold         → refuse
projected point outside window         → refuse
target behind camera / out of frustum  → refuse
raycast hit != target block (occluded) → refuse
post-action world diff != expected     → fail (verification), with evidence
```

Acceptance gate:

- One recorded refuse-with-reason (or verification-fail-with-evidence) per class
  on a real client — a real-client refusal sample matrix, not a blind click.
- Only when this is live is P0 actually done.

## Per-slice validation

On the Mac, per slice: `cargo fmt --check && cargo check && cargo test &&
git diff --check`, plus the slice's live-client smoke with run ids recorded. The
Fabric mod is Java/external, validated by a live telemetry sample, not cargo.
(The planning sandbox is Linux and cannot build the macOS crates.) This note is
docs-only and needs no cargo.

## Deferred decision — the perception fork (do NOT start in these slices)

After live MC-2/3/4 closure, choose one. Recorded here as **observations, not
started work**:

- **Option A — measurement-first.** Open MC-6 (spatial dataset recorder) with its
  FIRST consumer being a 2.5D-baseline measurement: keyframe-cache pose/occlusion
  error vs the mod's raycast + matrix ground-truth, swept across resource-pack
  texture richness (rich → flat-color → repetitive). The result becomes the
  empirical open-gate for whether MC-7 / 3DGS is ever needed. Also tighten P0 doc
  §8's "3D apps that do not expose truth" to API-denied / streamed surfaces
  (closed games, remote/streamed 3D), explicitly excluding script-exposed editors
  (Blender / Unity / Unreal), which drop to the API rung.
- **Option B — 3DGS** (MC-7, offline inspect artifact first per §8), pending an
  owner feasibility / compute check. Open dependency: a 3DGS difficulty trial plus
  external compute availability.

Both stay parked behind live P0 closure. §8's standing discipline holds: 3DGS is
not load-bearing for modded MC (raycast + depth is the stronger, cheaper truth
signal); do not pre-commit; let the "no truth source" second scenario pull it in.

## What to avoid next (unchanged from CLAUDE.md / the handoff)

- No third action-result schema beside `ActionResolverDecision` /
  `InputActionResult`.
- No Minecraft nouns graduated to core.
- No widening into a multi-slice refactor; no drive-by cleanup.
- No Mineflayer / MCP action path (KG2). Sidecar stays read-only truth + verifier.
- Do not start the MC-6 / MC-7 fork inside the live-closure slices.

## Fast restart checklist

Re-read before editing: `CLAUDE.md`, `AGENTS.md`,
`2026-06-14-auv-3d-minecraft-spatial-skill-p0.md`,
`2026-06-15-minecraft-series-handoff.md`. Start at Slice 1; finish, report, stop
for owner selection.
