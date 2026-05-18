use std::collections::BTreeSet;

use super::super::*;

pub(crate) fn parse_app_selector(raw: &str) -> AuvResult<AppSelector> {
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    return Err("app selector cannot be empty".to_string());
  }

  Ok(AppSelector {
    raw: trimmed.to_string(),
    bundle_id: if looks_like_bundle_identifier(trimmed) {
      Some(trimmed.to_string())
    } else {
      None
    },
    app_name_hint: if looks_like_bundle_identifier(trimmed) {
      None
    } else {
      Some(trimmed.to_string())
    },
  })
}

pub(crate) fn resolve_app_ref(
  snapshot: &ObservedWindowSnapshot,
  selector: &AppSelector,
) -> AuvResult<ResolvedAppRef> {
  if let Some(bundle_id) = selector.bundle_id.as_deref() {
    let exact_bundle_matches = snapshot
      .windows
      .iter()
      .filter(|window| window.owner_bundle_id.eq_ignore_ascii_case(bundle_id))
      .collect::<Vec<_>>();
    if !exact_bundle_matches.is_empty() {
      return Ok(build_resolved_app_ref(
        selector,
        Some(bundle_id.to_string()),
        &exact_bundle_matches,
        "bundle-id-exact",
      ));
    }

    if !snapshot.frontmost_app_bundle_id.is_empty()
      && snapshot
        .frontmost_app_bundle_id
        .eq_ignore_ascii_case(bundle_id)
      && !snapshot.frontmost_app_name.trim().is_empty()
    {
      let frontmost_name_matches = snapshot
        .windows
        .iter()
        .filter(|window| window.app_name == snapshot.frontmost_app_name)
        .collect::<Vec<_>>();
      if !frontmost_name_matches.is_empty() {
        return Ok(build_resolved_app_ref(
          selector,
          Some(bundle_id.to_string()),
          &frontmost_name_matches,
          "frontmost-bundle-fallback",
        ));
      }
    }
  }

  if let Some(app_name_hint) = selector.app_name_hint.as_deref() {
    let exact_name_matches = snapshot
      .windows
      .iter()
      .filter(|window| window.app_name.eq_ignore_ascii_case(app_name_hint))
      .collect::<Vec<_>>();
    if !exact_name_matches.is_empty() {
      return Ok(build_resolved_app_ref(
        selector,
        first_non_empty_bundle_id(&exact_name_matches),
        &exact_name_matches,
        "app-name-exact",
      ));
    }

    let heuristic_name_matches = snapshot
      .windows
      .iter()
      .filter(|window| app_contains_window(app_name_hint, &window.app_name))
      .collect::<Vec<_>>();
    if !heuristic_name_matches.is_empty() {
      return Ok(build_resolved_app_ref(
        selector,
        first_non_empty_bundle_id(&heuristic_name_matches),
        &heuristic_name_matches,
        "app-name-heuristic",
      ));
    }
  }

  Err(format!(
    "could not resolve a visible app reference for selector {:?}",
    selector.raw
  ))
}

pub(crate) fn resolve_window_ref(
  snapshot: &ObservedWindowSnapshot,
  resolved_app: &ResolvedAppRef,
) -> AuvResult<WindowRef> {
  let mut candidate_windows = snapshot
    .windows
    .iter()
    .filter(|window| window_matches_resolved_app(window, resolved_app))
    .collect::<Vec<_>>();
  if candidate_windows.is_empty() {
    return Err(format!(
      "could not resolve a visible window for selector {:?} via {}",
      resolved_app.selector.raw, resolved_app.match_strategy
    ));
  }

  if candidate_windows
    .iter()
    .any(|window| is_substantial_window(window))
  {
    candidate_windows.retain(|window| is_substantial_window(window));
  }

  let window = candidate_windows
    .into_iter()
    .max_by_key(|window| {
      (
        if window.layer == 0 { 1 } else { 0 },
        if is_substantial_window(window) { 1 } else { 0 },
        if !window.title.trim().is_empty() {
          1
        } else {
          0
        },
        window_area(window),
      )
    })
    .ok_or_else(|| {
      format!(
        "could not choose a preferred visible window for selector {:?}",
        resolved_app.selector.raw
      )
    })?;

  Ok(window.to_window_ref())
}

fn build_resolved_app_ref(
  selector: &AppSelector,
  resolved_bundle_id: Option<String>,
  windows: &[&ObservedWindow],
  match_strategy: &str,
) -> ResolvedAppRef {
  let resolved_app_name = windows
    .iter()
    .max_by_key(|window| {
      (
        if !window.title.trim().is_empty() {
          1
        } else {
          0
        },
        window_area(window),
      )
    })
    .map(|window| window.app_name.clone())
    .or_else(|| selector.app_name_hint.clone())
    .unwrap_or_else(|| selector.raw.clone());

  let owner_pids = windows
    .iter()
    .map(|window| window.owner_pid)
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();

  ResolvedAppRef {
    selector: selector.clone(),
    resolved_bundle_id,
    resolved_app_name,
    owner_pids,
    match_strategy: match_strategy.to_string(),
  }
}

fn first_non_empty_bundle_id(windows: &[&ObservedWindow]) -> Option<String> {
  windows.iter().find_map(|window| {
    (!window.owner_bundle_id.trim().is_empty()).then(|| window.owner_bundle_id.clone())
  })
}

fn window_matches_resolved_app(window: &ObservedWindow, resolved_app: &ResolvedAppRef) -> bool {
  if let Some(bundle_id) = resolved_app.resolved_bundle_id.as_deref() {
    if !window.owner_bundle_id.trim().is_empty() {
      return window.owner_bundle_id.eq_ignore_ascii_case(bundle_id);
    }
  }

  if resolved_app.owner_pids.contains(&window.owner_pid) {
    return true;
  }

  window
    .app_name
    .eq_ignore_ascii_case(&resolved_app.resolved_app_name)
}

fn is_substantial_window(window: &ObservedWindow) -> bool {
  window.bounds.width >= 160 && window.bounds.height >= 120
}
