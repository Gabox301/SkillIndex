//! Mirrors prompt.test.ts — one test per `it(...)` block.

use skillindex::prompt::{
    GroupState, MultiSelectOptions, compute_viewport_start, group_selection_state, multi_select,
    toggle_group_selection,
};
use std::io::{self, IsTerminal};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Dummy {
    name: String,
}

// ── multiSelect ────────────────────────────────────────────────────

#[test]
fn throws_when_initial_selected_length_does_not_match_items_length() {
    let items = vec![Dummy { name: "a".into() }];
    let opts = MultiSelectOptions {
        label_fn: Box::new(|d: &Dummy, _| d.name.clone()),
        initial_selected: Some(vec![true, false]),
        ..Default::default()
    };
    let res = multi_select(items, opts);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("initialSelected"));
}

#[test]
fn returns_all_items_when_stdin_is_not_a_tty() {
    // When stdin is not a TTY (as under `cargo test`), multi_select returns all.
    if io::stdin().is_terminal() {
        return;
    }
    let items = vec![Dummy { name: "a".into() }, Dummy { name: "b".into() }];
    let opts = MultiSelectOptions {
        label_fn: Box::new(|d: &Dummy, _| d.name.clone()),
        ..Default::default()
    };
    let res = multi_select(items.clone(), opts).unwrap();
    assert_eq!(res, items);
}

// ── groupSelectionState ─────────────────────────────────────────────

#[test]
fn returns_all_when_every_member_is_selected() {
    assert_eq!(
        group_selection_state(&[true, true, true], &[0, 1, 2]),
        GroupState::All
    );
}

#[test]
fn returns_none_when_no_member_is_selected() {
    assert_eq!(
        group_selection_state(&[false, false, false], &[0, 1, 2]),
        GroupState::None
    );
}

#[test]
fn returns_partial_when_members_are_mixed() {
    assert_eq!(
        group_selection_state(&[true, false, true], &[0, 1, 2]),
        GroupState::Partial
    );
}

#[test]
fn only_considers_the_given_member_indices() {
    // Indices 0,2 belong to the group; index 1 (selected) is outside it.
    assert_eq!(
        group_selection_state(&[false, true, false], &[0, 2]),
        GroupState::None
    );
    assert_eq!(
        group_selection_state(&[true, false, true], &[0, 2]),
        GroupState::All
    );
}

#[test]
fn returns_none_for_an_empty_group() {
    assert_eq!(group_selection_state(&[true, true], &[]), GroupState::None);
}

// ── toggleGroupSelection ────────────────────────────────────────────

#[test]
fn clears_the_group_when_all_members_are_selected() {
    let mut selected = vec![true, true, true];
    toggle_group_selection(&mut selected, &[0, 1, 2]);
    assert_eq!(selected, vec![false, false, false]);
}

#[test]
fn selects_the_whole_group_when_some_members_are_off() {
    let mut selected = vec![true, false, true];
    toggle_group_selection(&mut selected, &[0, 1, 2]);
    assert_eq!(selected, vec![true, true, true]);
}

#[test]
fn selects_the_whole_group_when_none_are_selected() {
    let mut selected = vec![false, false, false];
    toggle_group_selection(&mut selected, &[0, 1, 2]);
    assert_eq!(selected, vec![true, true, true]);
}

#[test]
fn only_touches_the_given_member_indices() {
    let mut selected = vec![true, true, false, false];
    // Group covers indices 2,3; index 0,1 must be untouched.
    toggle_group_selection(&mut selected, &[2, 3]);
    assert_eq!(selected, vec![true, true, true, true]);
}

// ── computeViewportStart ────────────────────────────────────────────

#[test]
fn returns_0_when_everything_fits_in_the_viewport() {
    assert_eq!(compute_viewport_start(4, 5, 10, 1, 0), 0);
}

#[test]
fn keeps_start_at_0_when_the_cursor_is_near_the_top() {
    assert_eq!(compute_viewport_start(0, 20, 5, 1, 0), 0);
    assert_eq!(compute_viewport_start(1, 20, 5, 1, 0), 0);
}

#[test]
fn slides_down_to_keep_the_cursor_visible_with_margin() {
    // cursor 10, height 5, margin 1 => start = 10 - 5 + 1 + 1 = 7
    assert_eq!(compute_viewport_start(10, 20, 5, 1, 0), 7);
}

#[test]
fn slides_up_to_keep_the_cursor_visible_with_margin() {
    // cursor 3 from a scrolled window => start = cursor - margin = 2
    assert_eq!(compute_viewport_start(3, 20, 5, 1, 7), 2);
}

#[test]
fn clamps_to_the_last_full_window_at_the_bottom() {
    // total 20, height 5 => max_start = 15
    assert_eq!(compute_viewport_start(19, 20, 5, 1, 0), 15);
}

#[test]
fn does_not_move_when_the_cursor_stays_within_the_window_and_margin() {
    // window [7,11], cursor 9 is comfortably inside => start unchanged
    assert_eq!(compute_viewport_start(9, 20, 5, 1, 7), 7);
}
