use crate::{PlaylistSidebarItem, PlaylistSidebarProjection, SidebarSection, SidebarSectionKind};
use auv_view::normalize_identity;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidebarState {
  /// A reconstructed NetEase sidebar section is available.
  Present,
  /// The caller knows the sidebar is not available in this view.
  Absent,
  /// Reconstruction ran, but did not identify a known sidebar section.
  Unknown,
}

/// Read-only sidebar facade backed by a reconstructed playlist sidebar projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SidebarView {
  state: SidebarState,
  projection: Option<PlaylistSidebarProjection>,
  playlist_lookup: Vec<PlaylistLookupEntry>,
}

impl SidebarView {
  /// Build a view for a caller-proven absent sidebar.
  pub fn absent() -> Self {
    Self {
      state: SidebarState::Absent,
      projection: None,
      playlist_lookup: Vec::new(),
    }
  }

  /// Build a view when the sidebar was not reconstructed by this observation.
  pub fn unknown() -> Self {
    Self {
      state: SidebarState::Unknown,
      projection: None,
      playlist_lookup: Vec::new(),
    }
  }

  /// Build a sidebar view from reconstructed sidebar data.
  pub fn from_projection(projection: PlaylistSidebarProjection) -> Self {
    let playlist_lookup = playlist_lookup(&projection);
    let state =
      if projection.sections.iter().any(is_known_sidebar_section) || !playlist_lookup.is_empty() {
        SidebarState::Present
      } else {
        SidebarState::Unknown
      };

    Self {
      state,
      playlist_lookup,
      projection: Some(projection),
    }
  }

  /// Return the sidebar availability state derived for this view.
  pub fn state(&self) -> SidebarState {
    self.state
  }

  /// Whether this view has a known reconstructed NetEase sidebar section.
  pub fn exists(&self) -> bool {
    self.state == SidebarState::Present
  }

  /// Find the first created/favorite playlist whose normalized label contains `keyword`.
  pub fn find_playlist(&self, keyword: &str) -> Option<&PlaylistSidebarItem> {
    let needle = normalize_identity(keyword);
    if needle.is_empty() {
      return None;
    }

    let projection = self.projection.as_ref()?;
    self
      .playlist_lookup
      .iter()
      .find(|entry| entry.normalized_label.contains(needle.as_str()))
      .and_then(|entry| {
        projection
          .sections
          .get(entry.section_index)
          .and_then(|section| section.items.get(entry.item_index))
      })
  }

  /// Return created/favorite playlists whose normalized labels contain `keyword`.
  ///
  /// `keyword == None` returns every playlist item in playlist collection sections.
  pub fn playlists(&self, keyword: Option<&str>) -> Vec<PlaylistRef<'_>> {
    let needle = keyword.map(normalize_identity);
    let Some(projection) = self.projection.as_ref() else {
      return Vec::new();
    };

    self
      .playlist_lookup
      .iter()
      .filter(|entry| {
        needle
          .as_ref()
          .is_none_or(|needle| entry.normalized_label.contains(needle.as_str()))
      })
      .filter_map(|entry| {
        let section = projection.sections.get(entry.section_index)?;
        let item = section.items.get(entry.item_index)?;
        Some(PlaylistRef { section, item })
      })
      .collect()
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PlaylistLookupEntry {
  section_index: usize,
  item_index: usize,
  normalized_label: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaylistRef<'a> {
  pub section: &'a SidebarSection,
  pub item: &'a PlaylistSidebarItem,
}

fn playlist_lookup(projection: &PlaylistSidebarProjection) -> Vec<PlaylistLookupEntry> {
  let has_playlist_collection = projection
    .sections
    .iter()
    .any(|section| is_playlist_collection(section.kind));

  projection
    .sections
    .iter()
    .enumerate()
    .filter(|(_, section)| {
      is_playlist_collection(section.kind)
        || (!has_playlist_collection && section.kind == SidebarSectionKind::Unknown)
    })
    .flat_map(|(section_index, section)| {
      section
        .items
        .iter()
        .enumerate()
        .map(move |(item_index, item)| PlaylistLookupEntry {
          section_index,
          item_index,
          normalized_label: normalize_identity(&item.label),
        })
    })
    .collect()
}

fn is_known_sidebar_section(section: &SidebarSection) -> bool {
  section.kind != SidebarSectionKind::Unknown
}

fn is_playlist_collection(kind: SidebarSectionKind) -> bool {
  matches!(
    kind,
    SidebarSectionKind::MyPlaylists | SidebarSectionKind::FavoritePlaylists
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{PlaylistSidebarItem, SidebarSection, SidebarSectionKind};
  use auv_view::Confidence;

  #[test]
  fn exists_when_projection_has_known_sidebar_playlist_section() {
    let view = SidebarView::from_projection(projection(vec![playlist_section(
      SidebarSectionKind::MyPlaylists,
      vec![],
    )]));

    assert_eq!(view.state(), SidebarState::Present);
    assert!(view.exists());
  }

  #[test]
  fn find_playlist_reads_projection_and_returns_matching_item_anchor() {
    let view = SidebarView::from_projection(projection(vec![playlist_section(
      SidebarSectionKind::MyPlaylists,
      vec![
        playlist_item("daily-mix", "Daily Mix", Some("anchor-daily")),
        playlist_item("workout", "Workout", None),
      ],
    )]));

    let item = view.find_playlist("daily").expect("playlist match");

    assert_eq!(item.id, "daily-mix");
    assert_eq!(item.anchor_id.as_deref(), Some("anchor-daily"));
  }

  #[test]
  fn absent_sidebar_does_not_match_playlist() {
    let view = SidebarView::absent();

    assert_eq!(view.state(), SidebarState::Absent);
    assert!(!view.exists());
    assert!(view.find_playlist("daily").is_none());
  }

  #[test]
  fn unknown_sidebar_does_not_claim_absence() {
    let view = SidebarView::unknown();

    assert_eq!(view.state(), SidebarState::Unknown);
    assert!(!view.exists());
    assert!(view.find_playlist("daily").is_none());
  }

  #[test]
  fn unknown_section_items_are_playlist_fallback_when_header_is_not_visible() {
    let view = SidebarView::from_projection(projection(vec![SidebarSection {
      id: "section.unassigned".to_string(),
      kind: SidebarSectionKind::Unknown,
      label: None,
      items: vec![playlist_item(
        "future-garage",
        "我喜欢的风格 | Future Garage",
        Some("anchor-future"),
      )],
    }]));

    let item = view
      .find_playlist("future garage")
      .expect("unassigned playlist row should match");

    assert_eq!(view.state(), SidebarState::Present);
    assert!(view.exists());
    assert_eq!(item.id, "future-garage");
    assert_eq!(item.anchor_id.as_deref(), Some("anchor-future"));
  }

  #[test]
  fn non_playlist_sections_do_not_satisfy_playlist_search() {
    let view = SidebarView::from_projection(projection(vec![SidebarSection {
      id: "feature-nav".to_string(),
      kind: SidebarSectionKind::FeatureNav,
      label: Some("推荐".to_string()),
      items: vec![playlist_item("daily-route", "Daily", Some("anchor-nav"))],
    }]));

    assert!(view.exists());
    assert!(view.find_playlist("daily").is_none());
  }

  fn projection(sections: Vec<SidebarSection>) -> PlaylistSidebarProjection {
    PlaylistSidebarProjection { sections }
  }

  fn playlist_section(kind: SidebarSectionKind, items: Vec<PlaylistSidebarItem>) -> SidebarSection {
    SidebarSection {
      id: "playlist-section".to_string(),
      kind,
      label: Some("我的歌单".to_string()),
      items,
    }
  }

  fn playlist_item(id: &str, label: &str, anchor_id: Option<&str>) -> PlaylistSidebarItem {
    PlaylistSidebarItem {
      id: id.to_string(),
      label: label.to_string(),
      section_hint: None,
      confidence: Confidence::High,
      candidate_id: None,
      anchor_id: anchor_id.map(str::to_string),
    }
  }
}
