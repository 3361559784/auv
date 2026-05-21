// SpanTree.jsx — indented span tree with status sigils + timing bar.

function spanGlyph(status) {
  const T = VIEWER_T;
  if (status === "running") return { g: "●", c: T.running, pulse: true };
  if (status === "ok")      return { g: "●", c: T.validated };
  if (status === "error")   return { g: "×", c: T.failed };
  if (status === "unset")   return { g: "○", c: T.fg3 };
  return { g: "·", c: T.fg3 };
}

function depthOf(spans, id, cache = {}) {
  if (id == null) return -1;
  if (cache[id] != null) return cache[id];
  const s = spans.find(x => x.id === id);
  if (!s || s.parent == null) return (cache[id] = 0);
  return (cache[id] = depthOf(spans, s.parent, cache) + 1);
}

function SpanTree({ spans, selectedSpanId, onSelect }) {
  const T = VIEWER_T;
  // Compute the longest duration for the timing bar normalization (live = use max).
  const totalSecs = spans.reduce((m, s) => {
    const n = parseFloat(s.t);
    return isFinite(n) ? Math.max(m, n) : m;
  }, 1);
  return (
    <div style={{ flex: 1, overflow: "auto", background: T.shell }}>
      <div style={{
        height: 28, position: "sticky", top: 0,
        background: T.shell2, borderBottom: `1px solid ${T.shellLine}`,
        display: "flex", alignItems: "center",
        padding: "0 16px", gap: 12,
        fontFamily: T.fontUI, fontSize: 10, fontWeight: 600,
        textTransform: "uppercase", letterSpacing: 0.8, color: T.fg3,
      }}>
        <span style={{ width: 14 }}/>
        <span style={{ flex: "0 0 300px" }}>span · name / step_id</span>
        <span style={{ flex: "0 0 70px" }}>status</span>
        <span style={{ flex: "0 0 70px" }}>dur</span>
        <span style={{ flex: 1 }}>timing</span>
      </div>
      {spans.map(s => {
        const d = depthOf(spans, s.id);
        const g = spanGlyph(s.status);
        const selected = s.id === selectedSpanId;
        const dur = parseFloat(s.t);
        const pct = isFinite(dur) ? Math.max(2, (dur / totalSecs) * 100) : 0;
        // mock bar offsets: cumulative-ish; we just shift later spans visually.
        const offsetPct = Math.min(60, (spans.indexOf(s) * 5));
        return (
          <button
            key={s.id}
            onClick={() => onSelect(s.id)}
            style={{
              width: "100%", textAlign: "left",
              background: selected ? T.shell3 : "transparent",
              borderLeft: `2px solid ${selected ? T.brand : "transparent"}`,
              border: 0, color: T.fg, cursor: "pointer",
              padding: "7px 16px",
              borderBottom: `1px solid ${T.shellLine}`,
              display: "flex", alignItems: "center", gap: 12,
              fontFamily: T.fontMono, fontSize: 12.5,
            }}>
            <span style={{ width: 14, color: g.c, display: "inline-block",
              animation: g.pulse ? "auv-pulse 1.2s linear infinite" : "none" }}>{g.g}</span>
            <span style={{ flex: "0 0 300px", paddingLeft: d * 16, color: T.fg, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
              <span style={{ color: T.brandSoft }}>{s.name}</span>
              {s.attrs && s.attrs.step_id ? <span style={{ color: T.fg3 }}>  step_id={s.attrs.step_id}</span> : null}
              {s.attrs && s.attrs.command_id ? <span style={{ color: T.fg3 }}>  {s.attrs.command_id}</span> : null}
            </span>
            <span style={{ flex: "0 0 70px", color: g.c, fontSize: 11 }}>
              {s.status === "running" ? "running" : s.status === "ok" ? "ok" : s.status === "error" ? "error" : "unset"}
            </span>
            <span style={{ flex: "0 0 70px", color: T.fg2 }}>{s.t}</span>
            <span style={{ flex: 1, height: 8, position: "relative", background: T.shell2, borderRadius: 1 }}>
              <span style={{
                position: "absolute", top: 0, bottom: 0,
                left: `${offsetPct}%`, width: `${pct}%`,
                background: g.c, opacity: s.status === "unset" ? 0.18 : 0.85,
                borderRadius: 1,
              }}/>
            </span>
          </button>
        );
      })}
    </div>
  );
}

Object.assign(window, { SpanTree });
