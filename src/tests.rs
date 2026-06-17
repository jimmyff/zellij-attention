// Tests build a `State` via `Default` then set only the few fields each case
// needs (and reassign them across phases to simulate runtime tab/pane reorders),
// which is clearer than full struct literals — opt out of that style lint here.
#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;
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
    state.notification_state.insert(pane_id, ntype);
}

#[test]
fn test_priority() {
    assert!(NotificationType::Attention.priority() > NotificationType::Working.priority());
    assert!(NotificationType::Working.priority() > NotificationType::Done.priority());
    assert_eq!(NotificationType::Attention.priority(), 3);
    assert_eq!(NotificationType::Working.priority(), 2);
    assert_eq!(NotificationType::Done.priority(), 1);
}

#[test]
fn test_strip_icons() {
    let state = State::default();
    assert_eq!(state.strip_icons("Tab 1 🚨"), "Tab 1");
    assert_eq!(state.strip_icons("Tab 1 ⏳"), "Tab 1");
    assert_eq!(state.strip_icons("Tab 1 ✅"), "Tab 1");
    assert_eq!(state.strip_icons("Tab 1 ⏳ ⏳"), "Tab 1");
    assert_eq!(state.strip_icons("Tab 1"), "Tab 1");
    assert_eq!(state.strip_icons(""), "");
}

#[test]
fn test_tab_name_has_icon() {
    let state = State::default();
    assert!(state.tab_name_has_icon("Tab 1 🚨"));
    assert!(state.tab_name_has_icon("Tab 1 ⏳"));
    assert!(state.tab_name_has_icon("Tab 1 ✅"));
    assert!(!state.tab_name_has_icon("Tab 1"));
    assert!(!state.tab_name_has_icon("⏳ Tab 1")); // icon not at end
}

#[test]
fn test_clean_stale_notifications_removes_old_pane_ids() {
    let mut state = State::default();
    add_notification(&mut state, 99, NotificationType::Working);
    state.panes = make_manifest(vec![(0, vec![make_pane(1, false, true)])]);

    assert!(state.clean_stale_notifications());
    assert!(state.notification_state.is_empty());
}

#[test]
fn test_clean_stale_skipped_when_panes_empty() {
    let mut state = State::default();
    add_notification(&mut state, 99, NotificationType::Working);

    assert!(!state.clean_stale_notifications());
    assert!(!state.notification_state.is_empty());
}

#[test]
fn test_get_tab_notification_state_skips_plugin_panes() {
    let mut state = State::default();
    state.panes = make_manifest(vec![(
        0,
        vec![
            make_pane(1, true, false), // plugin pane
            make_pane(2, false, true), // terminal pane
        ],
    )]);
    add_notification(&mut state, 1, NotificationType::Working);

    assert_eq!(state.get_tab_notification_state(0), None);

    add_notification(&mut state, 2, NotificationType::Done);
    assert_eq!(
        state.get_tab_notification_state(0),
        Some(NotificationType::Done)
    );
}

#[test]
fn test_tab_resolves_highest_priority() {
    let mut state = State::default();
    // Two Claude panes in one tab — the tab shows the higher-priority state.
    state.panes = make_manifest(vec![(
        0,
        vec![make_pane(1, false, false), make_pane(2, false, false)],
    )]);

    // [attention, done] -> attention
    add_notification(&mut state, 1, NotificationType::Attention);
    add_notification(&mut state, 2, NotificationType::Done);
    assert_eq!(
        state.get_tab_notification_state(0),
        Some(NotificationType::Attention)
    );

    // [working, done] -> working
    add_notification(&mut state, 1, NotificationType::Working);
    assert_eq!(
        state.get_tab_notification_state(0),
        Some(NotificationType::Working)
    );

    // [done, done] -> done
    add_notification(&mut state, 1, NotificationType::Done);
    assert_eq!(
        state.get_tab_notification_state(0),
        Some(NotificationType::Done)
    );
}

#[test]
fn test_check_and_clear_focus_clears_done() {
    let mut state = State::default();
    // Tab name must have an icon for focus-clear to proceed (reorder safety).
    state.tabs = vec![make_tab(0, "Tab 1 ✅", true)];
    state.panes = make_manifest(vec![(0, vec![make_pane(5, false, true)])]);
    add_notification(&mut state, 5, NotificationType::Done);

    assert!(state.check_and_clear_focus());
    assert!(state.notification_state.is_empty());
}

#[test]
fn test_live_status_persists_on_focus() {
    // Attention and working are live status — focusing the pane must NOT clear them.
    for (ntype, icon) in [
        (NotificationType::Attention, "🚨"),
        (NotificationType::Working, "⏳"),
    ] {
        let mut state = State::default();
        state.tabs = vec![make_tab(0, &format!("Tab 1 {}", icon), true)];
        state.panes = make_manifest(vec![(0, vec![make_pane(5, false, true)])]);
        add_notification(&mut state, 5, ntype);

        assert!(
            !state.check_and_clear_focus(),
            "{:?} should persist on focus",
            ntype
        );
        assert!(!state.notification_state.is_empty());
    }
}

#[test]
fn test_check_and_clear_focus_skips_without_icon() {
    let mut state = State::default();
    // Tab name has no icon — don't clear (protects against reorder race).
    state.tabs = vec![make_tab(0, "Tab 1", true)];
    state.panes = make_manifest(vec![(0, vec![make_pane(5, false, true)])]);
    add_notification(&mut state, 5, NotificationType::Done);

    assert!(!state.check_and_clear_focus());
    assert!(!state.notification_state.is_empty());
}

#[test]
fn test_done_while_focused_skips() {
    let mut state = State::default();
    state.tabs = vec![make_tab(0, "Tab 1", true)];
    state.panes = make_manifest(vec![(0, vec![make_pane(5, false, true)])]);

    // `done` arriving on the already-focused pane is pre-acknowledged → no marker painted.
    state.set_pane_notification(5, NotificationType::Done);
    assert!(state.notification_state.is_empty());
    assert_eq!(state.get_tab_notification_state(0), None);
}

#[test]
fn test_clear_removes_state() {
    let mut state = State::default();
    state.tabs = vec![make_tab(0, "Tab 1", true)];
    state.panes = make_manifest(vec![(0, vec![make_pane(5, false, false)])]);
    add_notification(&mut state, 5, NotificationType::Working);
    state.notified_tab_names.insert(5, "Tab 1".to_string());

    state.clear_pane_notification(5);
    assert!(state.notification_state.is_empty());
    assert!(state.notified_tab_names.is_empty());
}

#[test]
fn test_tab_reorder_skips_mismatched_tab_name() {
    let mut state = State::default();

    // Beta at pos 1 has a notification, recorded as tab "Beta"
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
    add_notification(&mut state, 2, NotificationType::Working);
    state.notified_tab_names.insert(2, "Beta".to_string());

    // After reorder: pane 2 is now at pos 2 but tab at pos 2 is "Tab #4"
    state.panes = make_manifest(vec![
        (0, vec![make_pane(1, false, false)]),
        (1, vec![make_pane(4, false, false)]),
        (2, vec![make_pane(2, false, false)]), // Beta's pane at Tab #4's position
        (3, vec![make_pane(3, false, true)]),
    ]);
    state.tabs = vec![
        make_tab(0, "Alpha", false),
        make_tab(1, "Beta ⏳", false), // stale tab data
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
    assert_eq!(
        state.get_tab_notification_state(2),
        Some(NotificationType::Working)
    );
}

#[test]
fn test_stale_icon_not_stripped_when_notification_expects_tab() {
    let mut state = State::default();

    // "Beta ⏳" at pos 1, notification expects tab "Beta"
    state.tabs = vec![make_tab(0, "Alpha", false), make_tab(1, "Beta ⏳", false)];
    state.panes = make_manifest(vec![
        (0, vec![make_pane(1, false, false)]),
        (1, vec![make_pane(2, false, false)]),
    ]);
    state.notified_tab_names.insert(2, "Beta".to_string());

    // "Beta ⏳" has an icon but the notification expects "Beta" — don't strip
    let base = state.strip_icons("Beta ⏳");
    assert!(state.notified_tab_names.values().any(|name| name == &base));
}
