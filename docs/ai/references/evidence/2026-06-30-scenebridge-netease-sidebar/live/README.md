# SceneBridge A3 live evidence (optional)

`proof_class: live`

Owner-labeled macOS evidence for A3e sign-off. Hermetic tests are the required
proof gate; this folder is optional unless the owner requests P8-full.

## Prerequisites

- NetEase Music installed and reachable via `auv-netease-music`
- macOS with driver permissions for window capture and input
- `AUV_NETEASE_VIEW_MEMORY=1` for the reacquire path

## Suggested transcript (redact secrets)

```bash
export AUV_NETEASE_VIEW_MEMORY=1
ARTIFACT_DIR=/tmp/auv-scenebridge-a3-live

# 1. Scan sidebar (writes playlist-scan-cache.json + view-memory-playlist_sidebar.json)
cargo run -p auv-netease-music -- playlist ls --json --artifact-dir "$ARTIFACT_DIR"

# 2. Select target (should skip rescan replay when memory + reacquire hit)
cargo run -p auv-netease-music -- playlist select "<playlist-label>" --json --artifact-dir "$ARTIFACT_DIR"
```

## Attachments

| File | Purpose |
| --- | --- |
| `transcript.txt` | Redacted command output |
| `playlist-select-result.json` | `reacquire` field populated when path succeeds |
| `view-memory-playlist_sidebar.json` | Copy from artifact dir after scan |

## Sign-off checklist

- [ ] Hermetic matrix green (`cargo test -p auv-view memory`)
- [ ] Live run labeled in this folder (if owner required)
- [ ] Owner approval to remove NOTICE / default-on feature gate
