use crate::framework::widget_system::commands::all_commands;
use crate::framework::widget_system::widgets::overlay_table::centered_area;
use crate::framework::widget_system::interaction::shortcuts::all_shortcuts;
use ratatui::layout::Rect;

const POPUP_MAX_HEIGHT: u16 = 24;

pub(crate) fn help_rows_len() -> usize {
    help_rows().len()
}

pub(crate) fn help_popup_area(area: Rect, rows: usize) -> Rect {
    centered_area(area, 90, rows, POPUP_MAX_HEIGHT)
}

pub(crate) struct HelpRow {
    pub(crate) kind: &'static str,
    pub(crate) trigger: String,
    pub(crate) description: &'static str,
}

pub(crate) fn help_rows() -> Vec<HelpRow> {
    let mut rows = Vec::new();
    for s in all_shortcuts() {
        rows.push(HelpRow {
            kind: "快捷键",
            trigger: s.keys.to_string(),
            description: s.description,
        });
    }
    for c in all_commands() {
        let trigger = if c.args.is_empty() {
            c.name.to_string()
        } else {
            format!("{} {}", c.name, c.args)
        };
        rows.push(HelpRow {
            kind: "命令",
            trigger,
            description: c.description,
        });
    }
    rows
}
