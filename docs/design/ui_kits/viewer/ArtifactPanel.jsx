// ArtifactPanel.jsx — right rail showing artifact list + selected preview.

function ArtifactIcon({ mime }) {
  const isImg = mime && mime.startsWith("image/");
  const isJson = mime === "application/json";
  const src = isImg
    ? "../../assets/icon-png.svg"
    : isJson
    ? "../../assets/icon-json.svg"
    : "../../assets/icon-bin.svg";
  return (
    <img
      src={src}
      alt=""
      style={{
        width: 28, height: 28, flex: "none",
        imageRendering: "pixelated",
        display: "block",
      }}
    />
  );
}

function ArtifactPreview({ a }) {
  const T = VIEWER_T;
  if (!a) {
    return (
      <div style={{
        padding: "30px 16px 24px",
        display: "flex", flexDirection: "column", alignItems: "center", gap: 12,
        color: T.fg3, fontFamily: T.fontSans, fontSize: 12,
        textAlign: "center",
      }}>
        <img
          src="../../assets/sprite-inspector.svg"
          alt=""
          style={{ width: 96, height: 112, imageRendering: "pixelated" }}
        />
        <div style={{ color: T.fg2 }}>Select an artifact to preview.</div>
        <div style={{ color: T.fg4, fontFamily: T.fontMono, fontSize: 10.5 }}>3 artifacts on this run</div>
      </div>
    );
  }
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10, padding: "14px 16px" }}>
      <div style={{
        display: "grid", gridTemplateColumns: "max-content 1fr", gap: "4px 14px",
        fontFamily: T.fontMono, fontSize: 11.5,
      }}>
        <span style={{ color: T.fg3 }}>role</span>     <span style={{ color: T.fg }}>{a.role}</span>
        <span style={{ color: T.fg3 }}>mime</span>     <span style={{ color: T.fg }}>{a.mime}</span>
        <span style={{ color: T.fg3 }}>path</span>     <span style={{ color: T.fg }}>{a.path}</span>
        <span style={{ color: T.fg3 }}>sha256</span>   <span style={{ color: T.fg }}>{a.sha}</span>
        <span style={{ color: T.fg3 }}>bytes</span>    <span style={{ color: T.fg }}>{a.bytes}</span>
        <span style={{ color: T.fg3 }}>span_id</span>  <span style={{ color: T.fg }}>{a.span}</span>
      </div>
      {/* Stand-in preview surface */}
      <div style={{
        marginTop: 6,
        height: 220,
        borderRadius: 4,
        border: `1px solid ${T.shellLine}`,
        background: a.mime === "application/json"
          ? T.shell3
          : `repeating-linear-gradient(45deg, ${T.shell2} 0 12px, ${T.shell3} 12px 24px)`,
        position: "relative", overflow: "hidden",
      }}>
        {a.mime === "application/json" ? (
          <pre style={{
            margin: 0, padding: 14,
            color: T.fg2, fontFamily: T.fontMono, fontSize: 11.5,
            lineHeight: 1.5, whiteSpace: "pre-wrap",
          }}>{`{
  "api_version": "auv.artifact.v1alpha1",
  "role": "ax.before",
  "subjectBundleId": "com.tencent.QQMusicMac",
  "windowRef": "win:0x83a1",
  "rootRole": "AXApplication",
  "childCount": 412,
  "notes": [
    "captured before resolve-ocr-anchor",
    "ax tree subset; full payload in artifacts/"
  ]
}`}</pre>
        ) : (
          <div style={{
            position: "absolute", inset: 0,
            display: "flex", alignItems: "center", justifyContent: "center",
            fontFamily: T.fontUI, fontSize: 11, color: T.fg2, letterSpacing: 0.4,
          }}>
            screenshot · {a.bytes}
          </div>
        )}
      </div>
    </div>
  );
}

function ArtifactPanel({ artifacts, selectedId, onSelect }) {
  const T = VIEWER_T;
  const selected = artifacts.find(a => a.id === selectedId);
  return (
    <div style={{
      width: 340, flex: "none",
      background: T.shell2,
      borderLeft: `1px solid ${T.shellLine}`,
      display: "flex", flexDirection: "column",
      overflow: "hidden",
    }}>
      <PaneHeader label="Artifacts · /artifacts" right={
        <span style={{ fontFamily: T.fontMono, fontSize: 11, color: T.fg3 }}>{artifacts.length}</span>
      }/>
      <div style={{ borderBottom: `1px solid ${T.shellLine}` }}>
        {artifacts.map(a => {
          const isActive = a.id === selectedId;
          return (
            <button
              key={a.id}
              onClick={() => onSelect(a.id)}
              style={{
                width: "100%", textAlign: "left",
                background: isActive ? T.shell3 : "transparent",
                border: 0,
                borderLeft: `2px solid ${isActive ? T.brand : "transparent"}`,
                color: T.fg, cursor: "pointer",
                padding: "10px 12px",
                display: "flex", alignItems: "center", gap: 10,
              }}
            >
              <ArtifactIcon mime={a.mime}/>
              <div style={{ display: "flex", flexDirection: "column", gap: 2, minWidth: 0 }}>
                <span style={{ fontFamily: T.fontMono, fontSize: 11.5, color: T.fg, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                  {a.role}
                </span>
                <span style={{ fontFamily: T.fontMono, fontSize: 10.5, color: T.fg3 }}>
                  {a.path.split("/").pop()}
                </span>
              </div>
              <div style={{ flex: 1 }}/>
              {a.live ? (
                <span style={{
                  fontFamily: T.fontUI, fontSize: 9.5, fontWeight: 500,
                  color: T.running, padding: "1px 6px",
                  border: `1px solid ${T.running}`, borderRadius: 2,
                }}>live</span>
              ) : null}
            </button>
          );
        })}
      </div>
      <div style={{ flex: 1, overflow: "auto" }}>
        <ArtifactPreview a={selected}/>
      </div>
    </div>
  );
}

Object.assign(window, { ArtifactPanel });
