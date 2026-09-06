pub type LabelFn<T> = dyn Fn(&T, usize) -> String;
pub type HintFn<T> = dyn Fn(&T, usize) -> String;
pub type GroupFn<T> = dyn Fn(&T) -> String;
pub type ShortcutFn<T> = dyn Fn(&[T]) -> Vec<bool>;

pub struct MultiSelectOptions<T> {
    pub label_fn: Box<LabelFn<T>>,
    pub hint_fn: Option<Box<HintFn<T>>>,
    pub group_fn: Option<Box<GroupFn<T>>>,
    pub initial_selected: Option<Vec<bool>>,
    pub shortcuts: Vec<Shortcut<T>>,
}

pub struct Shortcut<T> {
    pub key: char,
    pub label: String,
    pub func: Box<ShortcutFn<T>>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupState {
    All,
    None,
    Partial,
}

pub(crate) enum Row {
    Group { group: String, members: Vec<usize> },
    Item { index: usize },
}
