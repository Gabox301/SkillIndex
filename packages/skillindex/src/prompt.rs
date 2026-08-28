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
    if let Some(ref sel) = opts.initial_selected {
        if sel.len() != items.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "initialSelected length ({}) must match items length ({})",
                    sel.len(),
                    items.len()
                ),
            ));
        }
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
    let visible_group_count = if show_groups { g_count } else { 0 };
    let separator_count = if show_groups {
        g_count.saturating_sub(1)
    } else {
        0
    };

    let rendered_line_count = || items.len() + visible_group_count + separator_count + 1;

    let mut stdout = io::stdout();
    execute!(stdout, Hide)?;

    enable_raw_mode()?;

    #[allow(unused_assignments)]
    let mut rendered = false;

    let clear_rendered = |rendered: &mut bool, stdout: &mut io::Stdout| -> io::Result<()> {
        if *rendered {
            // Move up and clear
            let n = rendered_line_count();
            write!(stdout, "\x1b[{n}A\r\x1b[J")?;
            stdout.flush()?;
        }
        Ok(())
    };

    let draw = |stdout: &mut io::Stdout,
                selected: &[bool],
                cursor: usize,
                items: &[T],
                opts: &MultiSelectOptions<T>|
     -> io::Result<()> {
        let count = selected.iter().filter(|&&b| b).count();
        let mut last_group: Option<String> = None;
        let mut is_first_group = true;

        for i in 0..items.len() {
            if show_groups {
                if let Some(ref gf) = opts.group_fn {
                    let group = gf(&items[i]);
                    if last_group.as_ref() != Some(&group) {
                        if !is_first_group {
                            writeln!(stdout)?;
                        }
                        is_first_group = false;
                        last_group = Some(group.clone());
                        writeln!(stdout, "   {}", bold(&yellow(&group)))?;
                    }
                }
            }
            let pointer = if i == cursor {
                cyan("❯")
            } else {
                " ".to_string()
            };
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
            writeln!(stdout, "     {pointer} {check} {label}{hint_part}")?;
        }
        writeln!(stdout)?;

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
        hint_line.push_str(&dim(" alternar · "));
        hint_line.push_str(&white(&bold("[a]")));
        hint_line.push_str(&dim(" todas · "));
        hint_line.push_str(&shortcut_part);
        hint_line.push_str(&white(&bold("[enter]")));
        hint_line.push_str(&dim(&format!(" confirmar ({}/{})", count, items.len())));
        write!(stdout, "{hint_line}")?;
        stdout.flush()?;
        Ok(())
    };

    // Initial render
    {
        draw(&mut stdout, &selected, cursor, &items, &opts)?;
        rendered = true;
    }

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
                    clear_rendered(&mut rendered, &mut stdout)?;
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
                    selected[cursor] = !selected[cursor];
                    clear_rendered(&mut rendered, &mut stdout)?;
                    draw(&mut stdout, &selected, cursor, &items, &opts)?;
                }
                KeyCode::Char('a') => {
                    let all = selected.iter().all(|&b| b);
                    selected.fill(!all);
                    clear_rendered(&mut rendered, &mut stdout)?;
                    draw(&mut stdout, &selected, cursor, &items, &opts)?;
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
                        clear_rendered(&mut rendered, &mut stdout)?;
                        draw(&mut stdout, &selected, cursor, &items, &opts)?;
                    } else if c == 'k' {
                        cursor = cursor.checked_sub(1).unwrap_or(items.len() - 1);
                        clear_rendered(&mut rendered, &mut stdout)?;
                        draw(&mut stdout, &selected, cursor, &items, &opts)?;
                    } else if c == 'j' {
                        cursor = (cursor + 1) % items.len();
                        clear_rendered(&mut rendered, &mut stdout)?;
                        draw(&mut stdout, &selected, cursor, &items, &opts)?;
                    }
                }
                KeyCode::Up => {
                    cursor = cursor.checked_sub(1).unwrap_or(items.len() - 1);
                    clear_rendered(&mut rendered, &mut stdout)?;
                    draw(&mut stdout, &selected, cursor, &items, &opts)?;
                }
                KeyCode::Down => {
                    cursor = (cursor + 1) % items.len();
                    clear_rendered(&mut rendered, &mut stdout)?;
                    draw(&mut stdout, &selected, cursor, &items, &opts)?;
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

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Dummy {
        name: String,
        installed: bool,
    }

    #[test]
    fn initial_selected_length_mismatch_errors() {
        let items = vec![Dummy {
            name: "a".into(),
            installed: false,
        }];
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

    #[test]
    fn non_tty_returns_all() {
        // When stdin is not TTY (as in cargo test), multi_select returns all
        // We test with empty TTY simulation: since cargo test stdin is not TTY, it should return all
        if io::stdin().is_terminal() {
            // Skip — we are in TTY (unlikely in CI)
            return;
        }
        let items = vec![
            Dummy {
                name: "x".into(),
                installed: false,
            },
            Dummy {
                name: "y".into(),
                installed: false,
            },
        ];
        let opts = MultiSelectOptions {
            label_fn: Box::new(|d: &Dummy, _| d.name.clone()),
            ..Default::default()
        };
        let res = multi_select(items.clone(), opts).unwrap();
        assert_eq!(res, items);
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
}
