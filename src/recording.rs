use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use crate::model::{AuvResult, now_millis};
use crate::store::CanonicalRun;
use crate::trace::{
  ArtifactId, ArtifactRecordV1Alpha1, EventId, EventRecordV1Alpha1, RunId, RunRecordV1Alpha1,
  RunType, SpanId, SpanRecordV1Alpha1, TraceFailure, TraceState, TraceStatusCode,
};

pub type Attributes = BTreeMap<String, serde_json::Value>;

pub struct RunSpec {
  pub run_type: RunType,
  pub root_span_name: String,
  pub attributes: Attributes,
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use crate::trace::{
    EVENT_API_VERSION, EventRecordV1Alpha1, RUN_API_VERSION, RunId, RunRecordV1Alpha1, RunType,
    SPAN_API_VERSION, SpanId, SpanRecordV1Alpha1, TraceId, TraceState, TraceStatusCode,
  };

  use super::{BroadcastRunEventSink, MemoryRunEventSink, RecordingRun, SpanFinish, SpanRef};

  #[test]
  fn start_span_rejects_parent_from_another_run() {
    let mut run = recording_run("run_invalid_parent");
    let foreign_parent = SpanRef::new(SpanId::new("0000000000009999"));

    let error = run
      .start_span(&foreign_parent, span_record("auv.invalid.child"))
      .expect_err("foreign parent span should be rejected");

    assert!(error.contains("does not belong to run"));
  }

  #[test]
  fn broadcast_sink_replays_events_to_subscribers() {
    let sink = BroadcastRunEventSink::new(16);
    let mut receiver = sink.subscribe();
    let mut run = RecordingRun::new(
      RunRecordV1Alpha1 {
        api_version: RUN_API_VERSION.to_string(),
        run_id: RunId::new("run_broadcast_test"),
        trace_id: TraceId::new("00000000000000000000000000000001"),
        run_type: RunType::Command,
        state: TraceState::Running,
        status_code: TraceStatusCode::Unset,
        started_at_millis: 100,
        finished_at_millis: None,
        root_span_id: SpanId::new("0000000000000001"),
        attributes: Default::default(),
        summary: None,
        failure: None,
      },
      SpanRecordV1Alpha1 {
        api_version: SPAN_API_VERSION.to_string(),
        span_id: SpanId::new("0000000000000001"),
        parent_span_id: None,
        name: "auv.command".to_string(),
        state: TraceState::Running,
        status_code: TraceStatusCode::Unset,
        started_at_millis: 100,
        finished_at_millis: None,
        attributes: Default::default(),
        summary: None,
        failure: None,
      },
      Arc::new(sink),
    );

    run.record_event(EventRecordV1Alpha1 {
      api_version: EVENT_API_VERSION.to_string(),
      event_id: crate::trace::EventId::new("event_broadcast_test"),
      span_id: SpanId::new("0000000000000001"),
      name: "broadcast.event".to_string(),
      timestamp_millis: 101,
      attributes: Default::default(),
      message: None,
      artifact_ids: Vec::new(),
    });

    let first = receiver.try_recv().expect("root span should broadcast");
    assert!(matches!(first, super::RunStreamEvent::SpanStarted { .. }));
    let second = receiver
      .try_recv()
      .expect("recorded event should broadcast");
    assert!(matches!(
      second,
      super::RunStreamEvent::EventAppended { event, .. } if event.name == "broadcast.event"
    ));
  }

  #[test]
  fn finish_span_rejects_span_from_another_run() {
    let mut run = recording_run("run_invalid_finish");
    let foreign_span = SpanRef::new(SpanId::new("0000000000009998"));

    let error = run
      .finish_span(
        &foreign_span,
        SpanFinish {
          status_code: TraceStatusCode::Ok,
          summary: None,
          failure: None,
        },
      )
      .expect_err("foreign span should be rejected");

    assert!(error.contains("does not belong to run"));
  }

  fn recording_run(run_id: &str) -> RecordingRun {
    let root_span_id = SpanId::new("0000000000000001");
    RecordingRun::new(
      RunRecordV1Alpha1 {
        api_version: RUN_API_VERSION.to_string(),
        run_id: RunId::new(run_id),
        trace_id: TraceId::new("00000000000000000000000000000001"),
        run_type: RunType::Command,
        state: TraceState::Running,
        status_code: TraceStatusCode::Unset,
        started_at_millis: 100,
        finished_at_millis: None,
        root_span_id: root_span_id.clone(),
        attributes: Default::default(),
        summary: None,
        failure: None,
      },
      SpanRecordV1Alpha1 {
        api_version: SPAN_API_VERSION.to_string(),
        span_id: root_span_id,
        parent_span_id: None,
        name: "auv.command".to_string(),
        state: TraceState::Running,
        status_code: TraceStatusCode::Unset,
        started_at_millis: 100,
        finished_at_millis: None,
        attributes: Default::default(),
        summary: None,
        failure: None,
      },
      Arc::new(MemoryRunEventSink::new()),
    )
  }

  fn span_record(name: &str) -> SpanRecordV1Alpha1 {
    SpanRecordV1Alpha1 {
      api_version: SPAN_API_VERSION.to_string(),
      span_id: SpanId::new("0000000000000002"),
      parent_span_id: None,
      name: name.to_string(),
      state: TraceState::Running,
      status_code: TraceStatusCode::Unset,
      started_at_millis: 101,
      finished_at_millis: None,
      attributes: Default::default(),
      summary: None,
      failure: None,
    }
  }
}

impl RunSpec {
  pub fn new(run_type: RunType, root_span_name: impl Into<String>) -> Self {
    Self {
      run_type,
      root_span_name: root_span_name.into(),
      attributes: Attributes::new(),
    }
  }

  pub fn with_attributes(mut self, attributes: Attributes) -> Self {
    self.attributes = attributes;
    self
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpanRef {
  span_id: SpanId,
}

impl SpanRef {
  pub(crate) fn new(span_id: SpanId) -> Self {
    Self { span_id }
  }

  pub fn id(&self) -> &SpanId {
    &self.span_id
  }
}

pub struct RunFinish {
  pub status_code: TraceStatusCode,
  pub summary: Option<String>,
  pub failure: Option<String>,
}

pub struct SpanFinish {
  pub status_code: TraceStatusCode,
  pub summary: Option<String>,
  pub failure: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunStreamEvent {
  SpanStarted {
    run_id: RunId,
    span: SpanRecordV1Alpha1,
  },
  SpanFinished {
    run_id: RunId,
    span: SpanRecordV1Alpha1,
  },
  EventAppended {
    run_id: RunId,
    event: EventRecordV1Alpha1,
  },
  ArtifactCreated {
    run_id: RunId,
    artifact: ArtifactRecordV1Alpha1,
  },
  RunFinished {
    run_id: RunId,
    run: RunRecordV1Alpha1,
  },
}

impl RunStreamEvent {
  pub fn run_id(&self) -> &RunId {
    match self {
      Self::SpanStarted { run_id, .. }
      | Self::SpanFinished { run_id, .. }
      | Self::EventAppended { run_id, .. }
      | Self::ArtifactCreated { run_id, .. }
      | Self::RunFinished { run_id, .. } => run_id,
    }
  }
}

pub trait RunEventSink: Send + Sync {
  fn on_event(&self, event: RunStreamEvent);
}

#[derive(Clone)]
pub struct MemoryRunEventSink {
  events: Arc<Mutex<Vec<RunStreamEvent>>>,
}

impl MemoryRunEventSink {
  pub fn new() -> Self {
    Self {
      events: Arc::new(Mutex::new(Vec::new())),
    }
  }

  pub fn drain_for_test(&self) -> Vec<RunStreamEvent> {
    self
      .events
      .lock()
      .map(|events| events.clone())
      .unwrap_or_default()
  }
}

impl Default for MemoryRunEventSink {
  fn default() -> Self {
    Self::new()
  }
}

impl RunEventSink for MemoryRunEventSink {
  fn on_event(&self, event: RunStreamEvent) {
    if let Ok(mut events) = self.events.lock() {
      events.push(event);
    }
  }
}

#[derive(Clone)]
pub struct BroadcastRunEventSink {
  sender: broadcast::Sender<RunStreamEvent>,
}

impl BroadcastRunEventSink {
  pub fn new(capacity: usize) -> Self {
    let (sender, _) = broadcast::channel(capacity);
    Self { sender }
  }

  pub fn subscribe(&self) -> broadcast::Receiver<RunStreamEvent> {
    self.sender.subscribe()
  }
}

impl RunEventSink for BroadcastRunEventSink {
  fn on_event(&self, event: RunStreamEvent) {
    let _ = self.sender.send(event);
  }
}

pub struct RecordingRun {
  run: RunRecordV1Alpha1,
  spans: Vec<SpanRecordV1Alpha1>,
  events: Vec<EventRecordV1Alpha1>,
  artifacts: Vec<ArtifactRecordV1Alpha1>,
  event_sink: Arc<dyn RunEventSink>,
}

pub struct RecordedRun {
  pub snapshot: CanonicalRun,
}

impl RecordingRun {
  pub fn new(
    run: RunRecordV1Alpha1,
    root_span: SpanRecordV1Alpha1,
    event_sink: Arc<dyn RunEventSink>,
  ) -> Self {
    event_sink.on_event(RunStreamEvent::SpanStarted {
      run_id: run.run_id.clone(),
      span: root_span.clone(),
    });
    Self {
      run,
      spans: vec![root_span],
      events: Vec::new(),
      artifacts: Vec::new(),
      event_sink,
    }
  }

  pub fn id(&self) -> &RunId {
    &self.run.run_id
  }

  pub fn root_span(&self) -> SpanRef {
    SpanRef::new(self.run.root_span_id.clone())
  }

  pub fn start_span(
    &mut self,
    parent: &SpanRef,
    mut span: SpanRecordV1Alpha1,
  ) -> AuvResult<SpanRef> {
    if !self.has_span(parent.id()) {
      return Err(format!(
        "parent span {} does not belong to run {}",
        parent.id(),
        self.run.run_id
      ));
    }
    if self.has_span(&span.span_id) {
      return Err(format!(
        "span {} already belongs to run {}",
        span.span_id, self.run.run_id
      ));
    }
    span.parent_span_id = Some(parent.id().clone());
    let span_ref = SpanRef::new(span.span_id.clone());
    self.event_sink.on_event(RunStreamEvent::SpanStarted {
      run_id: self.run.run_id.clone(),
      span: span.clone(),
    });
    self.spans.push(span);
    Ok(span_ref)
  }

  pub fn finish_span(&mut self, span: &SpanRef, finish: SpanFinish) -> AuvResult<()> {
    if let Some(record) = self
      .spans
      .iter_mut()
      .find(|record| record.span_id == *span.id())
    {
      if record.state == TraceState::Ended {
        return Ok(());
      }
      record.state = TraceState::Ended;
      record.status_code = finish.status_code;
      record.finished_at_millis = Some(now_millis());
      record.summary = finish.summary;
      record.failure = finish.failure.map(|message| TraceFailure { message });
      self.event_sink.on_event(RunStreamEvent::SpanFinished {
        run_id: self.run.run_id.clone(),
        span: record.clone(),
      });
      return Ok(());
    }
    Err(format!(
      "span {} does not belong to run {}",
      span.id(),
      self.run.run_id
    ))
  }

  pub fn record_event(&mut self, event: EventRecordV1Alpha1) -> EventId {
    let event_id = event.event_id.clone();
    self.event_sink.on_event(RunStreamEvent::EventAppended {
      run_id: self.run.run_id.clone(),
      event: event.clone(),
    });
    self.events.push(event);
    event_id
  }

  pub fn record_artifact(&mut self, artifact: ArtifactRecordV1Alpha1) -> ArtifactId {
    let artifact_id = artifact.artifact_id.clone();
    self.event_sink.on_event(RunStreamEvent::ArtifactCreated {
      run_id: self.run.run_id.clone(),
      artifact: artifact.clone(),
    });
    self.artifacts.push(artifact);
    artifact_id
  }

  pub fn artifact_count(&self) -> usize {
    self.artifacts.len()
  }

  pub fn finish(
    mut self,
    status_code: TraceStatusCode,
    summary: Option<String>,
    failure: Option<TraceFailure>,
  ) -> RecordedRun {
    let finished_at_millis = now_millis();
    self.run.state = TraceState::Ended;
    self.run.status_code = status_code;
    self.run.finished_at_millis = Some(finished_at_millis);
    self.run.summary = summary;
    self.run.failure = failure;
    for span in &mut self.spans {
      if span.state == TraceState::Running {
        span.state = TraceState::Ended;
        span.status_code = status_code;
        span.finished_at_millis = Some(finished_at_millis);
        self.event_sink.on_event(RunStreamEvent::SpanFinished {
          run_id: self.run.run_id.clone(),
          span: span.clone(),
        });
      }
    }
    RecordedRun {
      snapshot: CanonicalRun {
        run: self.run,
        spans: self.spans,
        events: self.events,
        artifacts: self.artifacts,
      },
    }
  }

  fn has_span(&self, span_id: &SpanId) -> bool {
    self.spans.iter().any(|span| span.span_id == *span_id)
  }
}
