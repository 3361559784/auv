// Sidebar.jsx — run list (left nav, 280px)

function statusPill(status_code, state) {
  const T = VIEWER_T;
  if (state === "running") return { label: "running", color: T.running, line: T.runningLine, bg: T.runningSoft, pulse: true };
  if (status_code === "ok")    return { label: "ok",    color: T.validated, line: T.validatedLine, bg: T.validatedSoft };
  if (status_code === "error") return { label: "error", color: T.failed,    line: T.failedLine,    bg: T.failedSoft };
  return { label: "unset", color: T.frozen, line: T.frozenLine, bg: T.frozenSoft };
}

function Pill({ status_code, state, dark }) {
  const p = statusPill(status_code, state);
  const T = VIEWER_T;
  return (
    <span style={{
      display: "inline-flex", alignItems: "center", gap: 6,
      height: 20, padding: "0 8px 0 6px", borderRadius: 2,
      border: `1px solid ${dark ? p.color : p.line}`,
      background: dark ? "transparent" : p.bg,
      color: p.color,
      fontFamily: T.fontUI, fontSize: 11, fontWeight: 500,
    }}>
      <span style={{
        width: 7, height: 7, borderRadius: "50%", background: "currentColor",
        animation: p.pulse ? "auv-pulse 1.2s linear infinite" : "none",
      }}/>
      {p.label}
    </span>
  );
}

function RunTypeChip({ run_type }) {
  const T = VIEWER_T;
  return (
    <span style={{
      fontFamily: T.fontUI, fontSize: 10, fontWeight: 500,
      color: T.fg3, border: `1px solid ${T.shellLine}`,
      padding: "1px 6px", borderRadius: 2, letterSpacing: 0.4,
    }}>{run_type}</span>
  );
}

function midTrunc(s, head = 14, tail = 8) {
  if (s.length <= head + tail + 1) return s;
  return s.slice(0, head) + "…" + s.slice(-tail);
}

function Sidebar({ runs, activeId, onSelect }) {
  const T = VIEWER_T;
  return (
    <div style={{
      width: 320, flex: "none",
      background: T.shell2,
      borderRight: `1px solid ${T.shellLine}`,
      display: "flex", flexDirection: "column",
      overflow: "hidden",
    }}>
      <PaneHeader label="Runs · /runs" right={
        <span style={{ fontFamily: T.fontMono, fontSize: 11, color: T.fg3 }}>{runs.length}</span>
      }/>
      <div style={{ padding: "8px 12px", borderBottom: `1px solid ${T.shellLine}`, display: "flex", gap: 6, flexWrap: "wrap" }}>
        <FilterChip label="all" active/>
        <FilterChip label="running"/>
        <FilterChip label="error"/>
        <FilterChip label="execute"/>
        <FilterChip label="validate"/>
        <FilterChip label="probe"/>
      </div>
      <div style={{ overflow: "auto", flex: 1 }}>
        {runs.map(r => {
          const active = r.run_id === activeId;
          return (
            <button
              key={r.run_id}
              onClick={() => onSelect(r.run_id)}
              style={{
                width: "100%", textAlign: "left",
                background: active ? T.shell3 : "transparent",
                border: 0, borderBottom: `1px solid ${T.shellLine}`,
                borderLeft: `2px solid ${active ? T.brand : "transparent"}`,
                color: T.fg, cursor: "pointer",
                padding: "12px 14px",
                display: "flex", flexDirection: "column", gap: 6,
              }}
            >
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <Pill status_code={r.status_code} state={r.state} dark/>
                <RunTypeChip run_type={r.run_type}/>
                <span style={{ flex: 1 }}/>
                <span style={{ fontFamily: T.fontMono, fontSize: 11, color: T.fg3 }}>{r.duration}</span>
              </div>
              <div style={{ fontFamily: T.fontMono, fontSize: 12, color: T.fg, lineHeight: 1.35 }}>
                {midTrunc(r.run_id, 22, 8)}
              </div>
              <div style={{ fontFamily: T.fontSans, fontSize: 12, color: T.fg2, lineHeight: 1.4 }}>
                {r.summary}
              </div>
              <div style={{ fontFamily: T.fontMono, fontSize: 10, color: T.fg3, display: "flex", gap: 12 }}>
                <span>{r.spans} spans</span>
                <span>{r.artifacts} artifacts</span>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}

function FilterChip({ label, active }) {
  const T = VIEWER_T;
  return (
    <span style={{
      fontFamily: T.fontUI, fontSize: 11, fontWeight: 500,
      padding: "3px 8px", borderRadius: 2,
      border: `1px solid ${active ? T.brand : T.shellLine}`,
      background: active ? T.brand : "transparent",
      color: active ? "#fff" : T.fg2,
    }}>{label}</span>
  );
}

Object.assign(window, { Sidebar, Pill });
