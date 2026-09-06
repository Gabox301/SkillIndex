use super::types::{GroupFn, GroupState, Row};

pub fn group_count<T>(items: &[T], group_fn: Option<&GroupFn<T>>) -> usize {
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

pub fn toggle_group_selection(selected: &mut [bool], member_indices: &[usize]) {
    let next = group_selection_state(selected, member_indices) != GroupState::All;
    for &i in member_indices {
        selected[i] = next;
    }
}

pub fn compute_viewport_start(
    cursor: usize,
    total: usize,
    height: usize,
    margin: usize,
    prev_start: usize,
) -> usize {
    if height >= total {
        return 0;
    }
    let max_start = total - height;
    let m = margin.min((height - 1) / 2);
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

pub(crate) fn build_rows<T>(
    items: &[T],
    group_fn: Option<&GroupFn<T>>,
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
    fn group_count_computes_correctly() {
        let items = vec!["a", "b", "c"];
        let f: Box<dyn Fn(&&str) -> String> =
            Box::new(|s| if *s == "c" { "g2".into() } else { "g1".into() });
        let c = group_count(&items, Some(&f));
        assert_eq!(c, 2);
        let c0 = group_count(&items, None);
        assert_eq!(c0, 0);
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
}
