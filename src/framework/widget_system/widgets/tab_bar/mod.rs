use crate::framework::widget_system::interaction::text_utils::truncate_to_width_ellipsis_char;
use ratatui::layout::Rect;
use unicode_width::UnicodeWidthStr;

const MORE_LEFT_LABEL: &str = "«";
const MORE_RIGHT_LABEL: &str = "»";
const ADD_LABEL: &str = " + ";
pub(crate) const MAX_TAB_LABEL_WIDTH: usize = 20;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TabBarItemKind {
    Tab(usize),
    MoreLeft { target_pos: usize },
    MoreRight { target_pos: usize },
    Add,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TabBarItem {
    pub(crate) label: String,
    pub(crate) kind: TabBarItemKind,
    pub(crate) active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TabBarView {
    pub(crate) items: Vec<TabBarItem>,
}

pub(crate) fn build_tab_bar_view(labels: &[String], active_pos: usize, width: u16) -> TabBarView {
    let width = width as usize;
    if width == 0 || labels.is_empty() {
        return add_only_view(width);
    }
    let Some(tab_width) = tab_width_budget(width) else {
        return add_only_view(width);
    };
    let mut view = build_tab_items_view(labels, active_pos, tab_width);
    view.items.push(add_item());
    view
}

pub(crate) fn hit_test_tab_bar(x: u16, area: Rect, view: &TabBarView) -> Option<TabBarItemKind> {
    let mut cursor = area.x;
    for (i, item) in view.items.iter().enumerate() {
        let w = item.label.width() as u16;
        let next = cursor.saturating_add(w);
        if x >= cursor && x < next {
            return Some(item.kind.clone());
        }
        cursor = next;
        if i + 1 < view.items.len() {
            let sep_next = cursor.saturating_add(1);
            if x >= cursor && x < sep_next {
                return None;
            }
            cursor = sep_next;
        }
    }
    None
}

fn clamp_active_pos(len: usize, active_pos: usize) -> usize {
    if len == 0 {
        return 0;
    }
    active_pos.min(len - 1)
}

fn tab_width_budget(width: usize) -> Option<usize> {
    let add_width = ADD_LABEL.width();
    if width <= add_width {
        return None;
    }
    let budget = width.saturating_sub(add_width + 1);
    (budget > 0).then_some(budget)
}

fn add_only_view(width: usize) -> TabBarView {
    if width == 0 {
        return TabBarView { items: Vec::new() };
    }
    TabBarView {
        items: vec![add_item()],
    }
}

fn add_item() -> TabBarItem {
    TabBarItem {
        label: ADD_LABEL.to_string(),
        kind: TabBarItemKind::Add,
        active: false,
    }
}

fn build_tab_items_view(labels: &[String], active_pos: usize, width: usize) -> TabBarView {
    let active_pos = clamp_active_pos(labels.len(), active_pos);
    let truncated = truncate_labels(labels);
    if fits_all(&truncated, width) {
        return build_full_view(&truncated, active_pos);
    }
    build_windowed_view(&truncated, active_pos, width)
}

fn truncate_labels(labels: &[String]) -> Vec<String> {
    labels
        .iter()
        .map(|s| truncate_to_width_ellipsis_char(s, MAX_TAB_LABEL_WIDTH))
        .collect()
}

fn fits_all(labels: &[String], width: usize) -> bool {
    items_total_width(labels.iter().map(|s| s.as_str())) <= width
}

fn build_full_view(labels: &[String], active_pos: usize) -> TabBarView {
    let items = labels
        .iter()
        .enumerate()
        .map(|(i, label)| TabBarItem {
            label: label.clone(),
            kind: TabBarItemKind::Tab(i),
            active: i == active_pos,
        })
        .collect();
    TabBarView { items }
}

fn build_windowed_view(labels: &[String], active_pos: usize, width: usize) -> TabBarView {
    let Some((start, end)) = compute_window(labels, active_pos, width) else {
        return active_only_view(labels, active_pos, width);
    };
    let mut items = Vec::new();
    if start > 0 {
        items.push(TabBarItem {
            label: MORE_LEFT_LABEL.to_string(),
            kind: TabBarItemKind::MoreLeft {
                target_pos: start - 1,
            },
            active: false,
        });
    }
    for i in start..end {
        items.push(TabBarItem {
            label: labels.get(i).cloned().unwrap_or_default(),
            kind: TabBarItemKind::Tab(i),
            active: i == active_pos,
        });
    }
    if end < labels.len() {
        items.push(TabBarItem {
            label: MORE_RIGHT_LABEL.to_string(),
            kind: TabBarItemKind::MoreRight { target_pos: end },
            active: false,
        });
    }
    TabBarView { items }
}

fn active_only_view(labels: &[String], active_pos: usize, width: usize) -> TabBarView {
    let label = labels
        .get(active_pos)
        .map(|s| truncate_to_width_ellipsis_char(s, width))
        .unwrap_or_default();
    TabBarView {
        items: vec![TabBarItem {
            label,
            kind: TabBarItemKind::Tab(active_pos),
            active: true,
        }],
    }
}

fn compute_window(labels: &[String], active_pos: usize, width: usize) -> Option<(usize, usize)> {
    let len = labels.len();
    let mut start = active_pos;
    let mut end = active_pos + 1;
    if !window_fits(labels, start, end, width) {
        return None;
    }
    loop {
        let mut grown = false;
        if start > 0 && window_fits(labels, start - 1, end, width) {
            start -= 1;
            grown = true;
        }
        if end < len && window_fits(labels, start, end + 1, width) {
            end += 1;
            grown = true;
        }
        if !grown {
            break;
        }
    }
    Some((start, end))
}

fn window_fits(labels: &[String], start: usize, end: usize, width: usize) -> bool {
    let len = labels.len();
    let show_left = start > 0;
    let show_right = end < len;
    window_total_width(labels, start, end, show_left, show_right) <= width
}

fn window_total_width(
    labels: &[String],
    start: usize,
    end: usize,
    show_left: bool,
    show_right: bool,
) -> usize {
    let mut item_widths = Vec::new();
    if show_left {
        item_widths.push(MORE_LEFT_LABEL.width());
    }
    for i in start..end {
        let w = labels.get(i).map(|s| s.width()).unwrap_or(0);
        item_widths.push(w);
    }
    if show_right {
        item_widths.push(MORE_RIGHT_LABEL.width());
    }
    let items_width = item_widths.iter().sum::<usize>();
    let seps = item_widths.len().saturating_sub(1);
    items_width.saturating_add(seps)
}

fn items_total_width<'a, I>(items: I) -> usize
where
    I: Iterator<Item = &'a str>,
{
    let mut total = 0usize;
    let mut count = 0usize;
    for s in items {
        if count > 0 {
            total = total.saturating_add(1);
        }
        total = total.saturating_add(s.width());
        count = count.saturating_add(1);
    }
    total
}
