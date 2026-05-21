// EventsRail.jsx — events.jsonl tail + selected-span detail row above.

function SpanDetail({ span }) {
  const T = VIEWER_T;
  if (!span) {
    return (
      <div style={{
        padding: "20px",
        display: "flex", alignItems: "center", gap: 14,
        color: T.fg3,
        fontFamily: T.fontSans, fontSize: 13,
      }}>
        <img src="../../assets/sparkle.svg" alt="" style={{ width: 24, height: 24, imageRendering: "pixelated" }}/>
        Select a span to inspect its attributes.
      </div>
    );
  }
  return (
    <div style={{ padding: "14px 20px", color: T.fg, display: "flex", flexDirection: "column", gap: 8 }}>
      <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
        <span style={{ fontFamily: VIEWER_T.fontUI, fontSize: 10, letterSpacing: 0.8, textTransform: "uppercase", fontWeight: 600, color: T.fg3 }}>span</span>
        <span style={{ fontFamily: T.fontMono, fontSize: 13, color: T.brandSoft }}>{span.name}</span>
        <span style={{ fontFamily: T.fontMono, fontSize: 11, color: T.fg3 }}>span_id={span.id}</span>
      </div>
      <div style={{
        display: "grid",
        gridTemplateColumns: "max-content 1fr",
        rowGap: 4, columnGap: 16,
        fontFamily: T.fontMono, fontSize: 12,
      }}>
        {Object.entries(span.attrs || {}).map(([k, v]) => (
          <React.Fragment key={k}>
            <span style={{ color: T.fg3 }}>{k}</span>
            <span style={{ color: T.fg }}>{String(v)}</span>
          </React.Fragment>
        ))}
      </div>
    </div>
  );
}

function EventsRail({ events, span }) {
  const T = VIEWER_T;
  return (
    <div style={{
      flex: "0 0 320px",
      background: T.shell2,
      borderTop: `1px solid ${T.shellLine}`,
      display: "flex", flexDirection: "column",
    }}>
      <SpanDetail span={span}/>
      <PaneHeader label="Events · events.jsonl" right={
        <span style={{ fontFamily: VIEWER_T.fontMono, fontSize: 11, color: VIEWER_T.fg3 }}>
          {events.length} · tail
        </span>
      }/>
      <div style={{ overflow: "auto", flex: 1, padding: "6px 0" }}>
        {events.map((e, i) => (
          <div key={i} style={{
            display: "grid",
            gridTemplateColumns: "70px 160px 60px 1fr",
            gap: 12, padding: "4px 20px",
            fontFamily: T.fontMono, fontSize: 12, lineHeight: 1.45,
            background: e.live ? "rgba(31,125,140,0.08)" : "transparent",
          }}>
            <span style={{ color: T.fg3 }}>{e.t}</span>
            <span style={{ color: e.name.includes("failed") ? T.failed : e.name.includes("started") || e.name.includes("invoke") ? T.brandSoft : T.fg }}>{e.name}</span>
            <span style={{ color: T.fg3 }}>{e.span}</span>
            <span style={{ color: T.fg2 }}>{e.body}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

Object.assign(window, { EventsRail, SpanDetail });
