# `auv` Design System (vendored)

This directory is a **vendored copy** of the AUV Design System bundle that
was handed off from Claude Design (claude.ai/design) on **2026-05-21**.

The original handoff bundle's own README files are preserved verbatim:

- [`HANDOFF_README.md`](./HANDOFF_README.md) — Claude Design's
  per-handoff instructions to coding agents.
- [`SOURCE_README.md`](./SOURCE_README.md) — the design system's own
  README (product context, voice, visual foundations).
- [`SKILL.md`](./SKILL.md) — agent-skill metadata.

## Why vendor it?

The Rust runtime today consumes exactly **one** part of the design
system at compile time — the cursor sprite + brand pill in
`src/driver/macos/native/swift/Sources/AuvMacosNative/Overlay.swift`,
which ports rect data verbatim from `assets/cursor-auv.svg`.

The rest of the bundle is reference material for surfaces that **do not
exist in the repo yet** (the browser-based inspect viewer, an HTML CLI
mock, the broader component library). Keeping the canonical assets +
tokens in `docs/design/` means future implementations can match
pixel-perfectly against the same source the Overlay.swift sprite came
from, without re-fetching the original bundle.

## Layout

| Path | Purpose |
|---|---|
| `assets/` | Logo marks, cursor sprites, accents, sprite icons. All SVG (pixel-art, `viewBox="0 0 24 24"`, `shape-rendering="crispEdges"`). |
| `colors_and_type.css` | Single source of truth for design tokens — colors, type stack, spacing, radii, shadows, motion. |
| `preview/` | One-card-per-concept HTML previews (color cards, type cards, component cards). |
| `ui_kits/cli/` | High-fidelity HTML recreation of `auv-cli` terminal output. Not yet wired into the Rust CLI. |
| `ui_kits/viewer/` | Speculative recreation of the still-unbuilt browser viewer described in `docs/ai/references/2026-05-19-trace-run-inspect-design.md`. |

## What's implemented from this bundle

| Bundle element | Repo location | Status |
|---|---|---|
| `assets/cursor-auv.svg` rect data | `src/driver/.../Overlay.swift` (auvSprite) | ✅ done (Phase A) |
| `assets/cursor-you.svg` rect data | `src/driver/.../Overlay.swift` (youSprite) | ✅ ported (used by viewer mocks, not by the live overlay yet) |
| Brand cyan pill (`#009ba6`) | `src/driver/.../Overlay.swift` | ✅ done (Phase A) |
| `colors_and_type.css` tokens | this directory only | 📋 not yet consumed at runtime |
| `assets/cursor-auv-click.svg` (4-ray burst) | — | 📋 not yet ported; a future Overlay click-state could render it |
| `ui_kits/viewer/*` | — | 📋 will inform the browser viewer when it gets built (Phase C of design impl) |
| `ui_kits/cli/*` | — | 📋 reference for any future styled CLI output; the Rust CLI ships plain text today |

## Editing this directory

This is a vendored bundle. If the design system updates upstream, replace
the contents wholesale rather than hand-editing files here. The original
bundle ID was `Cnhoa_hmraSs_HJx96DFxw`; re-fetch from
`https://api.anthropic.com/v1/design/h/<id>` to obtain a fresh tarball.

The one file in this directory that is **not** part of the upstream
bundle is this `README.md` — it records the vendoring decision and the
implementation status. Keep it in sync with what the repo actually
consumes from the bundle.
