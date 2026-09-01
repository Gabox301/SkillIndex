use std::io::{self, IsTerminal, Write};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::ui::{bold, cyan, dim, green, white, yellow};

// ── Options ────────────────────────────────────────────────────────

#[allow(clippy::type_complexity)]
pub struct MultiSelectOptions<T> {
    pub label_fn: Box<dyn Fn(&T, usize) -> String>,
    pub hint_fn: Option<Box<dyn Fn(&T, usize) -> String>>,
    pub group_fn: Option<Box<dyn Fn(&T) -> String>>,
    pub initial_selected: Option<Vec<bool>>,
    pub shortcuts: Vec<Shortcut<T>>,
}

#[allow(clippy::type_complexity)]
pub struct Shortcut<T> {
    pub key: char,
    pub label: String,
    pub func: Box<dyn Fn(&[T]) -> Vec<bool>>,
}

impl<T> Default for MultiSelectOptions<T> {
    fn default() -> Self {
        Self {
            label_fn: Box::new(|_, _| String::new()),
            hint_fn: None,
            group_fn: None,
            initial_selected: None,
            shortcuts: Vec::new(),
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────

#[allow(clippy::type_complexity)]
fn group_count<T>(items: &[T], group_fn: Option<&Box<dyn Fn(&T) -> String>>) -> usize {
    let Some(f) = group_fn else { return 0 };
    let mut count = 0usize;
    let mut last: Option<String> = None;
    for item in items {
        let g = f(item);
        if last.as_ref() != Some(&g) {
            count += 1;
            last = Some(g);
        }
    }
    count
}

/// Selection state of a group given the selected flags of its member indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupState {
    All,
    None,
    Partial,
}

pub fn group_selection_state(selected: &[bool], member_indices: &[usize]) -> GroupState {
    if member_indices.is_empty() {
        return GroupState::None;
    }
    let mut any_on = false;
    let mut any_off = false;
    for &i in member_indices {
        if selected[i] {
            any_on = true;
        } else {
            any_off = true;
        }
    }
    if any_on && any_off {
        GroupState::Partial
    } else if any_on {
        GroupState::All
    } else {
        GroupState::None
    }
}

/// Smart group toggle: if every member is selected, clear them all; otherwise
/// select them all. Mutates `selected` in place for the given member indices.
pub fn toggle_group_selection(selected: &mut [bool], member_indices: &[usize]) {
    let next = group_selection_state(selected, member_indices) != GroupState::All;
    for &i in member_indices {
        selected[i] = next;
    }
}

/// Compute the new viewport start offset so the cursor stays visible with a
/// margin of context rows above/below. The window slides minimally: it only
/// moves when the cursor gets within `margin` rows of an edge, keeping prior
/// scroll position stable otherwise. The result is clamped to a valid start.
pub fn compute_viewport_start(
    cursor: usize,
    total: usize,
    height: usize,
    margin: usize,
    prev_start: usize,
) -> usize {
    // Everything fits: no scrolling.
    if height >= total {
        return 0;
    }
    let max_start = total - height;
    // Effective margin cannot exceed what the window can show on each side.
    let m = margin.min((height - 1) / 2);
    // Work in signed space to avoid usize underflow, then clamp.
    let cursor = cursor as isize;
    let height = height as isize;
    let m = m as isize;
    let mut start = prev_start as isize;
    if cursor - m < start {
        start = cursor - m;
    }
    if cursor + m > start + height - 1 {
        start = cursor - height + 1 + m;
    }
    start = start.clamp(0, max_start as isize);
    start as usize
}

/// A navigable row: either a group header (with its member item indices) or an item.
#[derive(Debug, Clone)]
enum Row {
    Group { group: String, members: Vec<usize> },
    Item { index: usize },
}

/// Build the ordered navigable rows. Group headers precede their items.
#[allow(clippy::type_complexity)]
fn build_rows<T>(
    items: &[T],
    group_fn: Option<&Box<dyn Fn(&T) -> String>>,
    show_groups: bool,
) -> Vec<Row> {
    match (show_groups, group_fn) {
        (true, Some(gf)) => {
            let mut rows: Vec<Row> = Vec::new();
            let mut last_group: Option<String> = None;
            let mut current_header: Option<usize> = None;
            for (i, item) in items.iter().enumerate() {
                let group = gf(item);
                if last_group.as_ref() != Some(&group) {
                    last_group = Some(group.clone());
                    rows.push(Row::Group {
                        group,
                        members: Vec::new(),
                    });
                    current_header = Some(rows.len() - 1);
                }
                if let Some(h) = current_header
                    && let Row::Group { members, .. } = &mut rows[h]
                {
                    members.push(i);
                }
                rows.push(Row::Item { index: i });
            }
            rows
        }
        _ => (0..items.len()).map(|index| Row::Item { index }).collect(),
    }
}

// ── Core multiSelect ───────────────────────────────────────────────

/// Interactive multi-select — mirrors `multiSelect` in ui.ts
/// - TTY raw mode, `❯` pointer, `◼/◻` checkboxes, grouped headers
/// - Shortcuts: `a` (all), plus custom `n`/`i` via `shortcuts`
/// - Navigation: ↑/↓, j/k, space toggle, enter confirm, Ctrl+C exit
/// - Non-TTY: returns all items
pub fn multi_select<T: Clone>(items: Vec<T>, opts: MultiSelectOptions<T>) -> io::Result<Vec<T>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    if let Some(ref sel) = opts.initial_selected
        && sel.len() != items.len()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "initialSelected length ({}) must match items length ({})",
                sel.len(),
                items.len()
            ),
        ));
    }

    if !io::stdin().is_terminal() {
        return Ok(items);
    }

    let mut selected = opts
        .initial_selected
        .clone()
        .unwrap_or_else(|| vec![true; items.len()]);
    let mut cursor: usize = 0;

    let g_count = group_count(&items, opts.group_fn.as_ref());
    let show_groups = g_count > 1;
    // Navigable rows: group headers interleaved with their items (when grouping),
    // otherwise one row per item. The cursor walks rows, not raw item indices.
    let rows = build_rows(&items, opts.group_fn.as_ref(), show_groups);

    // Viewport: when the list is taller than the terminal, only a sliding window
    // of rows is drawn so the block always fits and the cursor stays visible.
    const VIEWPORT_MARGIN: usize = 1;
    // Reserve lines for surrounding chrome (hint line + up/down indicators + some
    // breathing room already printed above the list).
    const RESERVED_ROWS: usize = 6;
    let terminal_rows = crossterm::terminal::size()
        .map(|(_, h)| h as usize)
        .unwrap_or(24);
    let viewport_height = rows
        .len()
        .min(terminal_rows.saturating_sub(RESERVED_ROWS))
        .max(3);
    let mut view_start: usize = 0;
    // Number of terminal lines the last draw() produced above the hint line.
    // Derived from the actual render so the rewind count can never drift.
    let mut last_drawn_lines: usize = 0;

    let mut stdout = io::stdout();
    execute!(stdout, Hide)?;

    enable_raw_mode()?;

    #[allow(unused_assignments)]
    let mut rendered = false;

    let clear_rendered =
        |rendered: &mut bool, stdout: &mut io::Stdout, lines: usize| -> io::Result<()> {
            if *rendered {
                write!(stdout, "\x1b[{lines}A\r\x1b[J")?;
                stdout.flush()?;
            }
            Ok(())
        };

    let group_check = |state: GroupState| -> String {
        match state {
            GroupState::All => green("◼"),
            GroupState::Partial => yellow("◧"),
            GroupState::None => dim("◻"),
        }
    };

    // Draws the visible window plus overflow indicators and the hint line.
    // Returns the number of lines written above the hint line, so the caller can
    // rewind exactly that many on the next redraw.
    let draw = |stdout: &mut io::Stdout,
                selected: &[bool],
                cursor: usize,
                view_start: usize,
                items: &[T],
                rows: &[Row],
                opts: &MultiSelectOptions<T>|
     -> io::Result<usize> {
        let count = selected.iter().filter(|&&b| b).count();
        let end = rows.len().min(view_start + viewport_height);
        let mut lines: usize = 0;

        // Overflow indicator: rows hidden above the window.
        if view_start > 0 {
            writeln!(stdout, "{}", dim(&format!("   ↑ {view_start} más")))?;
            lines += 1;
        }

        for (r, row) in rows.iter().enumerate().take(end).skip(view_start) {
            let pointer = if r == cursor {
                cyan("❯")
            } else {
                " ".to_string()
            };
            match row {
                Row::Group { group, members } => {
                    let state = group_selection_state(selected, members);
                    writeln!(
                        stdout,
                        "   {pointer} {} {}",
                        group_check(state),
                        bold(&yellow(group))
                    )?;
                }
                Row::Item { index } => {
                    let i = *index;
                    let check = if selected[i] {
                        green("◼")
                    } else {
                        dim("◻")
                    };
                    let label = (opts.label_fn)(&items[i], i);
                    let hint = opts
                        .hint_fn
                        .as_ref()
                        .map(|f| f(&items[i], i))
                        .unwrap_or_default();
                    let hint_part = if hint.is_empty() {
                        String::new()
                    } else {
                        format!("  {}", dim(&hint))
                    };
                    let indent = if show_groups { "       " } else { "     " };
                    writeln!(stdout, "{indent}{pointer} {check} {label}{hint_part}")?;
                }
            }
            lines += 1;
        }

        // Overflow indicator: rows hidden below the window.
        let below_count = rows.len() - end;
        if below_count > 0 {
            writeln!(stdout, "{}", dim(&format!("   ↓ {below_count} más")))?;
            lines += 1;
        }

        let shortcut_hints = opts
            .shortcuts
            .iter()
            .map(|s| {
                format!(
                    "{} {}",
                    white(&bold(&format!("[{}]", s.key))),
                    dim(&format!(" {}", s.label))
                )
            })
            .collect::<Vec<_>>()
            .join(&dim(" · "));
        let shortcut_part = if opts.shortcuts.is_empty() {
            String::new()
        } else {
            format!("{shortcut_hints}{}", dim(" · "))
        };

        let mut hint_line = String::new();
        hint_line.push_str("   ");
        hint_line.push_str(&white(&bold("[↑↓]")));
        hint_line.push_str(&dim(" mover · "));
        hint_line.push_str(&white(&bold("[espacio]")));
        hint_line.push_str(&dim(if show_groups {
            " alternar item/grupo · "
        } else {
            " alternar · "
        }));
        hint_line.push_str(&white(&bold("[a]")));
        hint_line.push_str(&dim(" todas · "));
        hint_line.push_str(&shortcut_part);
        hint_line.push_str(&white(&bold("[enter]")));
        hint_line.push_str(&dim(&format!(" confirmar ({}/{})", count, items.len())));
        write!(stdout, "{hint_line}")?;
        stdout.flush()?;
        Ok(lines)
    };

    let row_count = rows.len();

    // Recompute the viewport, clear the previous block, and redraw. Centralizes
    // the slide/clear/draw/line-count bookkeeping used on every key.
    macro_rules! redraw {
        () => {{
            view_start = compute_viewport_start(
                cursor,
                row_count,
                viewport_height,
                VIEWPORT_MARGIN,
                view_start,
            );
            clear_rendered(&mut rendered, &mut stdout, last_drawn_lines)?;
            last_drawn_lines = draw(
                &mut stdout,
                &selected,
                cursor,
                view_start,
                &items,
                &rows,
                &opts,
            )?;
            rendered = true;
        }};
    }

    // Initial render
    redraw!();

    loop {
        let event = event::read()?;
        if let Event::Key(key) = event {
            // Ctrl+C
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                disable_raw_mode()?;
                execute!(stdout, Show)?;
                writeln!(stdout)?;
                std::process::exit(0);
            }

            match key.code {
                KeyCode::Enter => {
                    disable_raw_mode()?;
                    clear_rendered(&mut rendered, &mut stdout, last_drawn_lines)?;
                    execute!(stdout, Show)?;
                    let result: Vec<T> = items
                        .into_iter()
                        .enumerate()
                        .filter(|(i, _)| selected[*i])
                        .map(|(_, v)| v)
                        .collect();
                    return Ok(result);
                }
                KeyCode::Char(' ') => {
                    match &rows[cursor] {
                        Row::Group { members, .. } => {
                            toggle_group_selection(&mut selected, members)
                        }
                        Row::Item { index } => selected[*index] = !selected[*index],
                    }
                    redraw!();
                }
                KeyCode::Char('a') => {
                    let all = selected.iter().all(|&b| b);
                    selected.fill(!all);
                    redraw!();
                }
                KeyCode::Char(c) => {
                    let mut handled = false;
                    for sc in &opts.shortcuts {
                        if sc.key == c {
                            let result = (sc.func)(&items);
                            for (i, v) in result.into_iter().enumerate() {
                                if i < selected.len() {
                                    selected[i] = v;
                                }
                            }
                            handled = true;
                            break;
                        }
                    }
                    if handled {
                        redraw!();
                    } else if c == 'k' {
                        cursor = cursor.checked_sub(1).unwrap_or(row_count - 1);
                        redraw!();
                    } else if c == 'j' {
                        cursor = (cursor + 1) % row_count;
                        redraw!();
                    }
                }
                KeyCode::Up => {
                    cursor = cursor.checked_sub(1).unwrap_or(row_count - 1);
                    redraw!();
                }
                KeyCode::Down => {
                    cursor = (cursor + 1) % row_count;
                    redraw!();
                }
                _ => {}
            }
        }
    }
}

/// Helper for tests: simulate shortcut `n` — select new-only (not installed)
pub fn shortcut_new_only<T, F>(items: &[T], is_installed: F) -> Vec<bool>
where
    F: Fn(&T) -> bool,
{
    items.iter().map(|it| !is_installed(it)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for the public API (multi_select, group_selection_state,
    // toggle_group_selection, shortcut_new_only) live in tests/prompt.rs,
    // mirroring prompt.test.ts. Only tests for private helpers stay here.

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Dummy {
        name: String,
        installed: bool,
    }

    #[test]
    fn group_count_computes_correctly() {
        let items = vec!["a", "b", "c"];
        let f: Box<dyn Fn(&&str) -> String> =
            Box::new(|s| if *s == "c" { "g2".into() } else { "g1".into() });
        let c = group_count(&items, Some(&f));
        assert_eq!(c, 2);
        let c0 = group_count(&items, None);
        assert_eq!(c0, 0);
    }

    // `shortcut_new_only` is a Rust-only helper with no TS mirror, so its test
    // stays as an internal unit test rather than in the mirrored tests/prompt.rs.
    #[test]
    fn shortcut_new_only_selects_new() {
        let items = vec![
            Dummy {
                name: "a".into(),
                installed: true,
            },
            Dummy {
                name: "b".into(),
                installed: false,
            },
            Dummy {
                name: "c".into(),
                installed: true,
            },
            Dummy {
                name: "d".into(),
                installed: false,
            },
        ];
        let sel = shortcut_new_only(&items, |d| d.installed);
        assert_eq!(sel, vec![false, true, false, true]);
    }
}
