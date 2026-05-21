// Prompt.jsx — $ prompt line + a small set of decorated row primitives.

function Prompt({ user = "moeru", host = "auv", cwd = "~/code/auv", command, args }) {
  const T = window.AUV_TOKENS;
  return (
    <div style={{ marginTop: 8 }}>
      <span style={{ color: T.validated }}>{user}@{host}</span>
      <span style={{ color: T.fg3 }}> </span>
      <span style={{ color: T.brand }}>{cwd}</span>
      <span style={{ color: T.fg3 }}> $ </span>
      <span style={{ color: T.fg }}>{command}</span>
      {args ? <span style={{ color: T.fg2 }}> {args}</span> : null}
    </div>
  );
}

function Comment({ children }) {
  const T = window.AUV_TOKENS;
  return <div style={{ color: T.fg3 }}>{children}</div>;
}

function Out({ children, color, indent = 0 }) {
  const T = window.AUV_TOKENS;
  return (
    <div style={{ color: color || T.fg, paddingLeft: indent * 14 }}>{children}</div>
  );
}

// "key: value" line with mono alignment via padding (not <table>).
function KV({ k, v, indent = 0, vColor }) {
  const T = window.AUV_TOKENS;
  return (
    <div style={{ paddingLeft: indent * 14 }}>
      <span style={{ color: T.fg3 }}>{k}:</span>
      <span style={{ color: vColor || T.fg }}> {v}</span>
    </div>
  );
}

// Sigil rows: "● validated   case-id"
function Sigil({ kind, label, id, note, indent = 0 }) {
  const T = window.AUV_TOKENS;
  const map = {
    validated: { glyph: "●", color: T.validated, label: "validated" },
    candidate: { glyph: "◐", color: T.candidate, label: "candidate" },
    boundary:  { glyph: "○", color: T.boundary,  label: "not-validated" },
    frozen:    { glyph: "■", color: T.frozen,    label: "phase-1-frozen" },
    failed:    { glyph: "×", color: T.failed,    label: "failed" },
    running:   { glyph: "●", color: T.running,   label: "running", pulse: true },
    ok:        { glyph: "✓", color: T.validated, label: "ok" },
    err:       { glyph: "✗", color: T.failed,    label: "error" },
  };
  const m = map[kind] || map.validated;
  return (
    <div style={{ paddingLeft: indent * 14 }}>
      <div style={{ display: "flex", gap: 10, alignItems: "baseline" }}>
        <span style={{
          color: m.color, width: 14, flex: "none",
          display: "inline-block",
          animation: m.pulse ? "auv-pulse 1.2s linear infinite" : "none",
        }}>{m.glyph}</span>
        <span style={{ color: m.color, width: 100, flex: "none" }}>{label || m.label}</span>
        <span style={{
          color: T.fg, flex: "1 1 auto", minWidth: 0,
          whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis",
        }}>{id}</span>
      </div>
      {note ? (
        <div style={{ paddingLeft: 124, color: T.fg3 }}>
          // {note}
        </div>
      ) : null}
    </div>
  );
}

function Blank() { return <div style={{ height: 4 }}/>; }

Object.assign(window, { Prompt, Comment, Out, KV, Sigil, Blank });
