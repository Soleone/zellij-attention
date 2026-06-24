use std::collections::{HashMap, HashSet};
use zellij_tile::prelude::*;

use crate::state::NotificationType;
use crate::State;

// Provide FFI stub so tests can link on native target
#[no_mangle]
pub extern "C" fn host_run_plugin_command() {}

fn make_tab(position: usize, name: &str, active: bool) -> TabInfo {
    TabInfo {
        position,
        name: name.to_string(),
        active,
        ..Default::default()
    }
}

fn make_pane(id: u32, is_plugin: bool, is_focused: bool) -> PaneInfo {
    PaneInfo {
        id,
        is_plugin,
        is_focused,
        ..Default::default()
    }
}

fn make_manifest(tab_panes: Vec<(usize, Vec<PaneInfo>)>) -> PaneManifest {
    let mut panes = HashMap::new();
    for (pos, p) in tab_panes {
        panes.insert(pos, p);
    }
    PaneManifest { panes }
}

fn add_notification(state: &mut State, pane_id: u32, ntype: NotificationType) {
    let mut set = HashSet::new();
    set.insert(ntype);
    state.notification_state.insert(pane_id, set);
}

fn pipe_event(name: &str) -> PipeMessage {
    PipeMessage {
        source: PipeSource::Keybind,
        name: name.to_string(),
        payload: None,
        args: std::collections::BTreeMap::new(),
        is_private: false,
    }
}

/// A focused, non-plugin terminal pane is required for focused-pane events.
fn state_with_focused_pane(pane_id: u32) -> State {
    let mut state = State::default();
    state.tabs = vec![make_tab(0, "work", true)];
    state.panes = make_manifest(vec![(0, vec![make_pane(pane_id, false, true)])]);
    state
}

fn notification_of(state: &State, pane_id: u32) -> Option<NotificationType> {
    state
        .notification_state
        .get(&pane_id)
        .and_then(|set| set.iter().copied().next())
}

#[test]
fn test_arm_flags_focused_pane_then_cmd_done_completes() {
    let mut state = state_with_focused_pane(7);

    // Keybind fires `arm` with no pane id; the plugin resolves the focused pane.
    state.pipe(pipe_event("zellij-attention::arm"));
    assert!(state.armed_panes.contains(&7));
    assert_eq!(notification_of(&state, 7), Some(NotificationType::Bash));

    // Shell pings cmd_done when the foreground command returns to the prompt.
    state.pipe(pipe_event("zellij-attention::cmd_done::7"));
    assert!(!state.armed_panes.contains(&7));
    assert_eq!(notification_of(&state, 7), Some(NotificationType::Completed));
}

#[test]
fn test_cmd_done_is_noop_when_pane_not_armed() {
    let mut state = state_with_focused_pane(7);

    state.pipe(pipe_event("zellij-attention::cmd_done::7"));
    assert!(state.armed_panes.is_empty());
    assert_eq!(notification_of(&state, 7), None);
}

#[test]
fn test_clear_cancels_pending_arm() {
    let mut state = state_with_focused_pane(7);
    state.pipe(pipe_event("zellij-attention::arm"));
    assert!(state.armed_panes.contains(&7));

    // Focusing the tab (or any clear) cancels the arm so a later command
    // completion does not resurrect a Completed icon.
    state.clear_pane_notification(7);
    assert!(!state.armed_panes.contains(&7));

    state.pipe(pipe_event("zellij-attention::cmd_done::7"));
    assert_eq!(notification_of(&state, 7), None);
}

#[test]
fn test_disarm_clears_without_completing() {
    let mut state = state_with_focused_pane(7);
    state.pipe(pipe_event("zellij-attention::arm"));

    state.pipe(pipe_event("zellij-attention::disarm"));
    assert!(!state.armed_panes.contains(&7));
    assert_eq!(notification_of(&state, 7), None);
}

#[test]
fn test_armed_pane_survives_active_tab_clear() {
    // Regression: arming the tab you are *currently viewing* must not be wiped
    // by the 0.5s active-tab auto-clear. Otherwise the pane is disarmed before
    // the shell's cmd_done arrives and the completion is lost.
    let mut state = State::default();
    // Active tab the user is sitting on; name already carries an icon so the
    // auto-clear guard (tab_name_has_icon) lets it proceed.
    state.tabs = vec![make_tab(0, "work \u{23F3}", true)];
    state.panes = make_manifest(vec![(0, vec![make_pane(7, false, true)])]);

    state.pipe(pipe_event("zellij-attention::arm"));
    assert!(state.armed_panes.contains(&7));
    assert_eq!(notification_of(&state, 7), Some(NotificationType::Bash));

    // Both auto-clear paths run on every 0.5s timer tick — neither may touch
    // the armed pane.
    assert!(!state.check_and_clear_active_tab());
    assert!(!state.check_and_clear_focus());
    assert!(state.armed_panes.contains(&7), "arm must survive active-tab clear");
    assert_eq!(notification_of(&state, 7), Some(NotificationType::Bash));

    // When the foreground command finishes, cmd_done completes it.
    state.pipe(pipe_event("zellij-attention::cmd_done::7"));
    assert!(!state.armed_panes.contains(&7));
    assert_eq!(notification_of(&state, 7), Some(NotificationType::Completed));
}

#[test]
fn test_arm_accepts_explicit_pane_id() {
    let mut state = state_with_focused_pane(7);

    // An explicit pane id overrides focus resolution (e.g. armed from elsewhere).
    state.pipe(pipe_event("zellij-attention::arm::42"));
    assert!(state.armed_panes.contains(&42));
    assert_eq!(notification_of(&state, 42), Some(NotificationType::Bash));
}

#[test]
fn test_strip_icons() {
    let state = State::default();
    assert_eq!(state.strip_icons("Tab 1 ⏳"), "Tab 1");
    assert_eq!(state.strip_icons("Tab 1 ✅"), "Tab 1");
    assert_eq!(state.strip_icons("Tab 1 ⏳ ⏳"), "Tab 1");
    assert_eq!(state.strip_icons("Tab 1"), "Tab 1");
    assert_eq!(state.strip_icons(""), "");
}

#[test]
fn test_tab_name_has_icon() {
    let state = State::default();
    assert!(state.tab_name_has_icon("Tab 1 ⏳"));
    assert!(state.tab_name_has_icon("Tab 1 ✅"));
    assert!(!state.tab_name_has_icon("Tab 1"));
    assert!(!state.tab_name_has_icon("⏳ Tab 1")); // icon not at end
}

#[test]
fn test_clean_stale_notifications_removes_old_pane_ids() {
    let mut state = State::default();
    add_notification(&mut state, 99, NotificationType::Waiting);
    state.panes = make_manifest(vec![(0, vec![make_pane(1, false, true)])]);

    assert!(state.clean_stale_notifications());
    assert!(state.notification_state.is_empty());
}

#[test]
fn test_clean_stale_skipped_when_panes_empty() {
    let mut state = State::default();
    add_notification(&mut state, 99, NotificationType::Waiting);

    assert!(!state.clean_stale_notifications());
    assert!(!state.notification_state.is_empty());
}

#[test]
fn test_get_tab_notification_state_skips_plugin_panes() {
    let mut state = State::default();
    state.panes = make_manifest(vec![
        (0, vec![
            make_pane(1, true, false),  // plugin pane
            make_pane(2, false, true),  // terminal pane
        ]),
    ]);
    add_notification(&mut state, 1, NotificationType::Waiting);

    assert_eq!(state.get_tab_notification_state(0), None);

    add_notification(&mut state, 2, NotificationType::Completed);
    assert_eq!(state.get_tab_notification_state(0), Some(NotificationType::Completed));
}

#[test]
fn test_check_and_clear_focus() {
    let mut state = State::default();
    // Tab name must have icon for focus-clear to proceed (reorder safety)
    state.tabs = vec![make_tab(0, "Tab 1 ⏳", true)];
    state.panes = make_manifest(vec![
        (0, vec![make_pane(5, false, true)]),
    ]);
    add_notification(&mut state, 5, NotificationType::Waiting);

    assert!(state.check_and_clear_focus());
    assert!(state.notification_state.is_empty());
}

#[test]
fn test_check_and_clear_focus_skips_without_icon() {
    let mut state = State::default();
    // Tab name has no icon — don't clear (protects against reorder race)
    state.tabs = vec![make_tab(0, "Tab 1", true)];
    state.panes = make_manifest(vec![
        (0, vec![make_pane(5, false, true)]),
    ]);
    add_notification(&mut state, 5, NotificationType::Waiting);

    assert!(!state.check_and_clear_focus());
    assert!(!state.notification_state.is_empty());
}

#[test]
fn test_tab_reorder_skips_mismatched_tab_name() {
    let mut state = State::default();

    // Beta at pos 1 has notification, recorded as tab "Beta"
    state.tabs = vec![
        make_tab(0, "Alpha", false),
        make_tab(1, "Beta ⏳", false),
        make_tab(2, "Gamma", true),
    ];
    state.panes = make_manifest(vec![
        (0, vec![make_pane(1, false, false)]),
        (1, vec![make_pane(2, false, false)]),
        (2, vec![make_pane(3, false, true)]),
    ]);
    add_notification(&mut state, 2, NotificationType::Waiting);
    state.notified_tab_names.insert(2, "Beta".to_string());

    // After reorder: pane 2 is now at pos 2 but tab at pos 2 is "Tab #4"
    state.panes = make_manifest(vec![
        (0, vec![make_pane(1, false, false)]),
        (1, vec![make_pane(4, false, false)]),
        (2, vec![make_pane(2, false, false)]),  // Beta's pane at Tab #4's position
        (3, vec![make_pane(3, false, true)]),
    ]);
    state.tabs = vec![
        make_tab(0, "Alpha", false),
        make_tab(1, "Beta ⏳", false),  // stale tab data
        make_tab(2, "Tab #4", true),
        make_tab(3, "Gamma", false),
    ];

    // Pane 2 is at pos 2 but tab is "Tab #4", not "Beta" — should skip
    assert_eq!(state.get_tab_notification_state(2), None);

    // After data stabilizes: pane 2 at pos 2, tab "Beta" at pos 2
    state.tabs = vec![
        make_tab(0, "Alpha", false),
        make_tab(1, "Tab #4", true),
        make_tab(2, "Beta ⏳", false),
        make_tab(3, "Gamma", false),
    ];

    // Now tab name matches — notification should be found
    assert_eq!(state.get_tab_notification_state(2), Some(NotificationType::Waiting));
}

#[test]
fn test_stale_icon_not_stripped_when_notification_expects_tab() {
    let mut state = State::default();

    // "Beta ⏳" at pos 1, notification expects tab "Beta"
    state.tabs = vec![
        make_tab(0, "Alpha", false),
        make_tab(1, "Beta ⏳", false),
    ];
    state.panes = make_manifest(vec![
        (0, vec![make_pane(1, false, false)]),
        (1, vec![make_pane(2, false, false)]),
    ]);
    state.notified_tab_names.insert(2, "Beta".to_string());

    // "Beta ⏳" has icon but notification expects "Beta" — don't strip
    let base = state.strip_icons("Beta ⏳");
    assert!(state.notified_tab_names.values().any(|name| name == &base));
}
