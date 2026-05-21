// Terminal.jsx — macOS Terminal.app-style window chrome + scrollback container
// Exports: Terminal, TabStrip

function TabStrip({ tabs, active, onSelect }) {
  const T = window.AUV_TOKENS;
  return (
    <div style={{
      display: "flex",
      alignItems: "stretch",
      background: "#1f2228",
      borderBottom: "1px solid #000",
      height: 30,
    }}>
      {tabs.map((tab, i) => {
        const isActive = i === active;
        return (
          <button
            key={tab.id}
            onClick={() => onSelect(i)}
            style={{
              flex: 1,
              background: isActive ? T.shell : "transparent",
              color: isActive ? T.fg : T.fg3,
              border: 0,
              borderRight: i < tabs.length - 1 ? "1px solid #000" : 0,
              borderTop: isActive ? `1px solid ${T.brand}` : "1px solid transparent",
              fontFamily: T.fontMono,
              fontSize: 11,
              fontWeight: 500,
              letterSpacing: 0.2,
              cursor: "pointer",
              padding: "0 14px",
              textAlign: "left",
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
            }}
            title={tab.label}
          >
            {tab.label}
          </button>
        );
      })}
    </div>
  );
}

function TrafficLights() {
  const dot = (bg) => (
    <span style={{
      width: 12, height: 12, borderRadius: "50%", background: bg,
      display: "inline-block",
      boxShadow: "inset 0 0 0 0.5px rgba(0,0,0,0.25)",
    }}/>
  );
  return (
    <div style={{ display: "flex", gap: 8, alignItems: "center", padding: "0 0 0 4px" }}>
      {dot("#ff5f57")}{dot("#febc2e")}{dot("#28c840")}
    </div>
  );
}

function Terminal({ title = "auv — bash — 132×40", tabs, active, onSelect, children, height = 540 }) {
  const T = window.AUV_TOKENS;
  return (
    <div style={{
      width: "100%",
      maxWidth: 1000,
      margin: "0 auto",
      borderRadius: 10,
      overflow: "hidden",
      background: T.shell,
      boxShadow: "0 24px 60px rgba(0,0,0,0.35), 0 6px 16px rgba(0,0,0,0.2)",
      border: "1px solid #000",
    }}>
      {/* titlebar */}
      <div style={{
        height: 28,
        background: "linear-gradient(#3a3d44, #2c2f35)",
        borderBottom: "1px solid #000",
        display: "flex",
        alignItems: "center",
        padding: "0 12px",
        position: "relative",
      }}>
        <TrafficLights />
        <div style={{
          position: "absolute",
          inset: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          pointerEvents: "none",
          color: "#d6d6d6",
          fontFamily: T.fontSans,
          fontSize: 12,
          fontWeight: 500,
        }}>{title}</div>
      </div>
      {/* tabs */}
      {tabs && tabs.length > 1 ? (
        <TabStrip tabs={tabs} active={active} onSelect={onSelect}/>
      ) : null}
      {/* scrollback */}
      <div style={{
        background: T.shell,
        color: T.fg,
        fontFamily: T.fontMono,
        fontSize: 12.5,
        lineHeight: 1.6,
        padding: "16px 20px 24px",
        height,
        overflow: "auto",
      }}>
        {children}
      </div>
    </div>
  );
}

Object.assign(window, { Terminal, TabStrip });
