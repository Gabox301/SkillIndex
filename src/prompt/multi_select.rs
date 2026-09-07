use std::io::{self, IsTerminal, Write};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::ui::{bold, brand_cyan, dim, green, white, yellow};

use super::helpers::{
    build_rows, compute_viewport_start, group_selection_state, toggle_group_selection,
};
use super::types::{GroupState, MultiSelectOptions, Row};

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

    let mut selected: Vec<bool> = opts
        .initial_selected
        .clone()
        .unwrap_or_else(|| vec![true; items.len()]);
    let mut cursor: usize = 0;

    let g_count: usize = crate::prompt::helpers::group_count(&items, opts.group_fn.as_deref());
    let show_groups: bool = g_count > 1;
    let rows: Vec<Row> = build_rows(&items, opts.group_fn.as_deref(), show_groups);

    const VIEWPORT_MARGIN: usize = 1;
    const RESERVED_ROWS: usize = 6;
    let terminal_rows: usize = crossterm::terminal::size()
        .map(|(_, h)| h as usize)
        .unwrap_or(24);
    let viewport_height: usize = rows
        .len()
        .min(terminal_rows.saturating_sub(RESERVED_ROWS))
        .max(3);
    let mut view_start: usize = 0;
    let mut last_drawn_lines: usize = 0;

    let mut stdout: io::Stdout = io::stdout();
    execute!(stdout, Hide)?;

    enable_raw_mode()?;

    let mut rendered: bool = false;

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

    let draw = |stdout: &mut io::Stdout,
                selected: &[bool],
                cursor: usize,
                view_start: usize,
                items: &[T],
                rows: &[Row],
                opts: &MultiSelectOptions<T>|
     -> io::Result<usize> {
        let count: usize = selected.iter().filter(|&&b| b).count();
        let end: usize = rows.len().min(view_start + viewport_height);
        let mut lines: usize = 0;

        if view_start > 0 {
            writeln!(stdout, "{}", dim(&format!("   ↑ {view_start} más")))?;
            lines += 1;
        }

        for (r, row) in rows.iter().enumerate().take(end).skip(view_start) {
            let pointer: String = if r == cursor {
                brand_cyan("❯")
            } else {
                " ".to_string()
            };
            match row {
                Row::Group { group, members } => {
                    let state: GroupState = group_selection_state(selected, members);
                    writeln!(
                        stdout,
                        "   {pointer} {} {}",
                        group_check(state),
                        bold(&yellow(group))
                    )?;
                }
                Row::Item { index } => {
                    let i: usize = *index;
                    let check: String = if selected[i] {
                        green("◼")
                    } else {
                        dim("◻")
                    };
                    let label: String = (opts.label_fn)(&items[i], i);
                    let hint: String = opts
                        .hint_fn
                        .as_ref()
                        .map(|f| f(&items[i], i))
                        .unwrap_or_default();
                    let hint_part: String = if hint.is_empty() {
                        String::new()
                    } else {
                        format!("  {}", dim(&hint))
                    };
                    let indent: &str = if show_groups { "       " } else { "     " };
                    writeln!(stdout, "{indent}{pointer} {check} {label}{hint_part}")?;
                }
            }
            lines += 1;
        }

        let below_count: usize = rows.len() - end;
        if below_count > 0 {
            writeln!(stdout, "{}", dim(&format!("   ↓ {below_count} más")))?;
            lines += 1;
        }

        let shortcut_hints: String = opts
            .shortcuts
            .iter()
            .map(|s: &crate::prompt::Shortcut<T>| {
                format!(
                    "{} {}",
                    white(&bold(&format!("[{}]", s.key))),
                    dim(&format!(" {}", s.label))
                )
            })
            .collect::<Vec<_>>()
            .join(&dim(" · "));
        let shortcut_part: String = if opts.shortcuts.is_empty() {
            String::new()
        } else {
            format!("{shortcut_hints}{}", dim(" · "))
        };

        let mut hint_line: String = String::new();
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

    let row_count: usize = rows.len();

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

    redraw!();

    loop {
        let event: Event = event::read()?;
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                continue;
            }
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
                    let all: bool = selected.iter().all(|&b| b);
                    selected.fill(!all);
                    redraw!();
                }
                KeyCode::Char(c) => {
                    let mut handled: bool = false;
                    for sc in &opts.shortcuts {
                        if sc.key == c {
                            let result: Vec<bool> = (sc.func)(&items);
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
