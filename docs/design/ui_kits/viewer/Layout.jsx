// Layout.jsx — Top bar, sidebar shell, content shell.

const VIEWER_T = {
  paper: "#f6f5f1", paper2: "#efeee8", paper3: "#e6e4dc", paperLine: "#d8d5cb",
  shell: "#0e1013", shell2: "#16181d", shell3: "#1e2127", shellLine: "#2a2e36",
  ink: "#15171a", ink2: "#2c2f34", ink3: "#5a5e66", ink4: "#8a8e96",
  fg: "#e7e5dd", fg2: "#b8b6ad", fg3: "#7a7972", fg4: "#4f4e49",
  brand: "#00c4d2", brandSoft: "#cff4f7", brandLine: "#8de1e8",
  validated: "#2f7d4f", validatedSoft: "#dff0e3", validatedLine: "#b6d8be",
  candidate: "#b46a14", candidateSoft: "#fbecd3", candidateLine: "#e8c990",
  boundary:  "#a73b41", boundarySoft: "#f7dcde",  boundaryLine: "#ebb1b4",
  frozen:    "#4a5462", frozenSoft: "#e3e7ec",   frozenLine: "#c3cad3",
  running:   "#1f7d8c", runningSoft: "#d3eaee",  runningLine: "#99ccd3",
  failed:    "#c0392b", failedSoft: "#fadcd7",   failedLine: "#f0a99e",
  fontMono:  '"JetBrains Mono", ui-monospace, "SF Mono", Menlo, Consolas, monospace',
  fontSans:  '"Geist", ui-sans-serif, system-ui, sans-serif',
  fontUI:    '"Geist Mono", "JetBrains Mono", ui-monospace, Menlo, Consolas, monospace',
};

function TopBar({ connection }) {
  const live = connection === "live";
  const T = VIEWER_T;
  return (
    <div style={{
      height: 44, flex: "none",
      background: T.shell,
      borderBottom: `1px solid ${T.shellLine}`,
      display: "flex", alignItems: "center",
      padding: "0 16px", gap: 14, color: T.fg,
    }}>
      <img src="../../assets/logo-mark.svg" alt="" style={{ width: 22, height: 22, imageRendering: "pixelated" }}/>
      <div style={{ fontFamily: T.fontMono, fontSize: 13, fontWeight: 500 }}>auv</div>
      <div style={{ fontFamily: T.fontMono, fontSize: 12, color: T.fg3 }}>/ inspect viewer</div>
      <img src="../../assets/sparkle.svg" alt="" style={{ width: 14, height: 14, imageRendering: "pixelated", opacity: 0.85 }}/>
      <div style={{ flex: 1 }}/>
      <div style={{
        display: "flex", alignItems: "center", gap: 6,
        height: 22, padding: "0 9px 0 7px", borderRadius: 2,
        background: live ? T.shell2 : T.shell2,
        border: `1px solid ${live ? T.running : T.failed}`,
        color: live ? T.running : T.failed,
        fontFamily: T.fontUI, fontSize: 12, fontWeight: 500,
      }}>
        <span style={{
          width: 7, height: 7, borderRadius: "50%", background: "currentColor",
          animation: live ? "auv-pulse 1.2s linear infinite" : "none",
        }}/>
        {live ? "live" : "disconnected"}
      </div>
      <div style={{ fontFamily: T.fontMono, fontSize: 11, color: T.fg2 }}>
        ws://127.0.0.1:8765/runs/run_1778947574511.../stream
      </div>
    </div>
  );
}

function Shell({ children }) {
  return (
    <div style={{
      width: "100%",
      height: "100vh",
      display: "flex", flexDirection: "column",
      background: VIEWER_T.shell,
      fontFamily: VIEWER_T.fontSans,
    }}>
      {children}
    </div>
  );
}

function PaneHeader({ label, right, dark = true }) {
  const T = VIEWER_T;
  return (
    <div style={{
      height: 32, flex: "none",
      display: "flex", alignItems: "center", gap: 10,
      padding: "0 16px",
      borderBottom: `1px solid ${dark ? T.shellLine : T.paperLine}`,
      background: dark ? T.shell2 : T.paper2,
    }}>
      <span style={{
        fontFamily: T.fontUI, fontSize: 10, letterSpacing: 0.8,
        textTransform: "uppercase", fontWeight: 600,
        color: dark ? T.fg3 : T.ink3,
      }}>{label}</span>
      <div style={{ flex: 1 }}/>
      {right}
    </div>
  );
}

Object.assign(window, { VIEWER_T, TopBar, Shell, PaneHeader });
