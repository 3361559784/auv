# AUV macOS Now-Playing Capability (`auv-media-macos`) v0 Design

Date: 2026-06-04

Status: v0 design spec, updated to the as-built architecture. A new leaf crate
`auv-media-macos` reads the macOS system now-playing state via the vendored
`mediaremote-adapter` (built from source), driven through Apple's
`/usr/bin/perl`. The crate is **lib + binary**: it exposes the capability as a
library, owns the agent-facing `now-playing-v0` output contract, and ships its
own `auv-now-playing` binary. The existing `auv-netease-music` CLI gains a
`now-playing` subcommand that **delegates** to the crate, so both front doors
emit one identical contract.

Audience: owner, reviewers, and any agent (Codex, Claude, others) implementing
or reviewing the now-playing capability.

## Purpose

An agent loop that drives a music app needs to read "what is playing right
now" without OCR-ing the player bar. Today AUV verifies playback only by
capturing and recognizing the on-screen now-playing region. That is visual,
app-specific, and brittle.

macOS aggregates now-playing state (title, artist, album, duration, elapsed
time, playback rate, owning app) for whatever app holds the system Now Playing
slot — the same data Control Center and the media keys use. This spec exposes
that state as a structured, agent-callable read.

## Why the obvious approach does not work (verified)

The only macOS API that returns *another* app's now-playing state is the
**private** `MediaRemote.framework`. The public `MPNowPlayingInfoCenter` only
exposes the calling process's own info.

Since **macOS 15.4**, MediaRemote now-playing reads are gated: per the
mediaremote-adapter project, only a process whose **bundle identifier starts
with `com.apple.`** is permitted to read it. This was verified empirically and
is the load-bearing fact of this design:

- A `dlopen` of MediaRemote from our **compiled, ad-hoc-signed binary** returns
  an **empty** dict (no error) on macOS 26.2.
- The same calls from `/usr/bin/swift` (identifier `com.apple.dt…`) or
  `/usr/bin/perl` (`com.apple.perl`) return full data.

> Lesson recorded: an early probe run as `swift probe.swift` returned data and
> produced a **false positive** — it executed inside the Apple-signed swift
> toolchain process. Feasibility for a shippable capability must be tested with
> a *compiled standalone binary*, never `swift`-script. We cannot sign our own
> binary as `com.apple.*`, so we must borrow an Apple platform binary as the
> vehicle.

The chosen vehicle is **`/usr/bin/perl`** (present on every macOS; no Swift
toolchain needed at runtime) driving the BSD-licensed **mediaremote-adapter**,
which loads a small `MediaRemoteAdapter.framework` and prints now-playing JSON.
Confirmed working on macOS 26.2 from a compiled binary at ~20 ms/read (warm).

## Source semantics (decided)

MediaRemote now-playing is **system-wide and app-agnostic**: it returns
whichever app currently owns the Now Playing slot (NetEase, Spotify, Music, a
browser tab — all identical). This capability does **not** filter to NetEase;
it reports whatever is playing and includes the owning app's
`source_bundle_id` so the caller can decide. The capability is therefore a
generic crate with its own `auv-now-playing` binary; the netease-music
`now-playing` subcommand is an additional convenience front door (that is the
existing agent-facing product CLI), not because the read is NetEase-specific.

## Crate placement and layout

One new workspace member: a **leaf, lib + binary, pure-Rust** crate. No
swift-bridge, no in-process FFI, no native static lib linked into the binary.

```text
crates/auv-media-macos/
  Cargo.toml               // one [[bin]]: auv-now-playing; deps: serde, serde_json, clap
  build.rs                 // cmake-builds the vendored framework, tars it into OUT_DIR
  vendor/
    mediaremote-adapter/   // git submodule, pinned to upstream release v0.7.6 (BSD-3)
  src/
    lib.rs                 // NowPlayingState, pure parse_get(), now_playing()
    adapter.rs             // (macOS) embed framework+script, unpack to cache, run perl
    output.rs              // now-playing-v0 contract type + JSON/human builders
    cli.rs                 // argv -> output mode, run() -> ExitCode (the binary)
    error.rs               // MediaError
    bin/
      auv-now-playing.rs   // thin main -> auv_media_macos::cli::run()
```

Build-time (`build.rs`, macOS only): runs `cmake` to build
`MediaRemoteAdapter.framework` from the submodule, then `tar`s the built bundle
into `OUT_DIR`. Off-macOS, `build.rs` is a no-op and the adapter module is not
compiled.

Runtime (`adapter.rs`, macOS only): the built framework tar is embedded via
`include_bytes!` and the perl driver via `include_str!`. On first use they are
unpacked to a content-keyed cache
(`~/Library/Caches/auv/mediaremote-adapter/<hash>/`, atomic rename), then the
read runs `/usr/bin/perl <script> <framework> get`. The binary is therefore
self-contained: it needs only stock `/usr/bin/perl`, no external file layout.

`Cargo.toml` depends only on `serde` + `serde_json` + `clap`. It does **not**
depend on `auv-driver-macos`, `auv-cli`, or `auv-netease-music` (leaf crate).
Registered in the root `Cargo.toml` `[workspace].members`.

Fresh checkouts must run `git submodule update --init --recursive`; `build.rs`
panics with that exact hint if the submodule is missing.

### Why this structure (recorded decisions)

- **Build adapter from source** (submodule + cmake), not a committed binary
  blob: reproducible, auditable, multi-arch, matches the Rust `-sys`-crate
  convention. The repo already needs a native toolchain (`auv-driver-macos`
  shells `swiftc`), so cmake is a peer ask, not a new burden.
- **Embed + unpack** (vs sibling files): yields a single self-contained binary
  that runs from anywhere and survives being moved.
- **Pin to a release tag** (`v0.7.6`): submodules pin a commit by construction;
  we pin the commit of an upstream *release* rather than a floating `main`.

## Capability API

```rust
pub struct NowPlayingState {
  pub present: bool,                 // an app owns the slot with valid content
  pub source_bundle_id: Option<String>,
  pub title: Option<String>,
  pub artist: Option<String>,
  pub album: Option<String>,
  pub duration_seconds: Option<f64>,
  pub elapsed_seconds: Option<f64>,
  pub playback_rate: Option<f64>,
  pub is_playing: bool,              // from the adapter's `playing` flag
  pub content_item_id: Option<String>,
}

pub fn now_playing() -> Result<NowPlayingState, MediaError>;
```

- `now_playing()` (macOS) runs the adapter `get` and feeds its JSON to a pure
  `parse_get(&str) -> Result<NowPlayingState, MediaError>`. The adapter emits
  the literal `null` when nothing valid is playing (→ idle `NowPlayingState`);
  otherwise an object whose mandatory keys are `bundleIdentifier`, `playing`,
  `title`. `artworkData` and other keys are intentionally ignored.
- **Non-macOS:** `now_playing()` returns `MediaError::Unsupported`.

The crate also owns the agent-facing contract and the binary entry:

```rust
// output.rs
pub const SCHEMA_VERSION: &str = "now-playing-v0";
pub struct NowPlayingOutput { /* schema_version + flattened NowPlayingState */ }
pub fn build_now_playing_output(state: &NowPlayingState) -> NowPlayingOutput;
pub fn render_human_summary(state: &NowPlayingState) -> String;

// cli.rs
pub fn run() -> std::process::ExitCode;
```

## CLI surface (two front doors, one contract)

Both surfaces parse the same flags and emit the identical `now-playing-v0`
contract built in `auv-media-macos::output`.

```text
# the crate's own binary
auv-now-playing [--json | --json-out <path>]

# the netease-music subcommand (delegates to the crate)
auv-netease-music now-playing [--json | --json-out <path>]   (auv-wyy = identical)
```

The netease subcommand calls `auv_media_macos::now_playing()` then the crate's
`build_now_playing_output` / `render_human_summary` — it does **not** reshape
the contract. On non-macOS it prints "only available on macOS" and exits
non-zero. `auv-netease-music` depends on `auv-media-macos` as a
`cfg(target_os = "macos")` dependency.

### Human output (default)

- Playing: `▶ <title> — <artist> [<album>]  (<source_bundle_id>)`
- Paused:  `⏸ <title> — <artist> [<album>]  (<source_bundle_id>)`
- Idle:    `Nothing playing`

(Absent / empty optional fields are omitted, not printed as empty brackets.)

## Output contract (agent-facing)

- `--json` / `--json-out` produce a stable object carrying
  `schema_version: "now-playing-v0"` plus the `NowPlayingState` fields. The
  contract type + builders live in `auv-media-macos::output` (crate-owned), so
  the binary and the netease subcommand emit byte-identical JSON.
- Exit codes:
  - `0` — the read completed, **including the nothing-playing case**
    (`present: false`). "Nothing playing" is state, not an error — consistent
    with the `playlist` contract.
  - non-zero — adapter/perl failure (perl missing, adapter non-zero exit,
    malformed JSON) or a non-macOS `Unsupported` result.
- An agent distinguishes "paused" from "idle" via `is_playing` + `present`, and
  the source app via `source_bundle_id` (it does not infer the app from the
  track text).

## Testing

Pure-Rust unit tests (no live media, no perl required):

- `parse_get`: `null` → idle; mapped object; paused (present, not playing);
  garbage → error (4 tests).
- `output`: `now-playing-v0` JSON carries schema version + fields; human
  summary playing / paused / idle / omitted-empty-fields (5 tests).

The live adapter read is environmental and macOS-gated; it is not a CI unit
test — its mechanism is proven by running the compiled binary. This mirrors how
existing live-driver procedures are gated while their pure logic is unit-tested.

## Validation

Behavior change, so on completion run: `cargo fmt --check`, `cargo check`,
`cargo test`, `git diff --check`, plus CLI smoke checks on **both** front doors
(`auv-now-playing` and `auv-netease-music now-playing`, human + `--json`, and
`--help` listing it) confirming identical `now-playing-v0` JSON.

## Non-goals (v0) and the agreed next slice

v0 is **read-only, one-shot**. Explicitly not in v0:

- artwork bytes (the adapter emits `artworkData`; we ignore it — a suppress-
  artwork flag to keep the pipe small is a possible follow-up);
- NetEase-specific filtering (source is reported, not gated);
- live-position extrapolation;
- change subscription / streaming (the adapter's `stream` exists; deferred).

**Agreed next slice — transport controls.** The same adapter exposes
`send`/`seek`/`shuffle`/`repeat`/`speed`. The owner approved adding **transport
+ seek** (play, pause, toggle, next, previous, seek) as a separate follow-up
slice with its own mini-design: a typed `MediaCommand` enum → MRCommand IDs and
`send_command(cmd) -> Result<(), MediaError>` (a plain `Result`, **not** a new
action-result schema). This is the "media subsystem" the standalone crate was
positioned to seed.

## Risks

- **Private framework via a borrowed vehicle.** The read depends on
  `/usr/bin/perl` remaining an Apple platform binary permitted to read
  MediaRemote, and on the private framework's behavior. Apple could close this
  (as it closed direct in-process access in 15.4). Any break surfaces as a
  `MediaError` (non-zero exit), never a silent wrong answer; the existing
  player-bar OCR path remains the durable fallback.
- **Vendored third-party dependency.** mediaremote-adapter (BSD-3) is pinned as
  a submodule at `v0.7.6`; bumping it is a manual, reviewable step. Its LICENSE
  is retained with the vendored source.
- **Build/runtime prerequisites.** Build needs `cmake` + the initialized
  submodule (clear panic otherwise). Runtime needs `/usr/bin/perl` (stock on
  macOS).
