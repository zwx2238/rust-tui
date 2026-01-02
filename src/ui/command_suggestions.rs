use crate::render::RenderTheme;
use crate::ui::commands::{
    CommandSuggestion, CommandSuggestionKind, command_has_args, command_suggestions_for_input,
};
use crate::ui::overlay_table::{
    OverlayTable, draw_overlay_table, header_style, row_at, visible_rows,
};
use crate::ui::selection_state::SelectionState;
use crate::ui::state::{App, Focus};
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};

pub(crate) fn refresh_command_suggestions(app: &mut App) {
    if app.focus != Focus::Input {
        clear_command_suggestions(app);
        return;
    }
    if app.busy || app.pending_code_exec.is_some() || app.pending_file_patch.is_some() {
        clear_command_suggestions(app);
        return;
    }
    let suggestions = current_input_suggestions(app);
    if suggestions.is_empty() {
        clear_command_suggestions(app);
        return;
    }
    app.command_suggestions = suggestions;
    app.command_select = SelectionState::default();
}

pub(crate) fn clear_command_suggestions(app: &mut App) {
    app.command_suggestions.clear();
    app.command_select = SelectionState::default();
}

pub(crate) fn command_suggestions_active(app: &App) -> bool {
    !app.command_suggestions.is_empty()
}

pub(crate) fn apply_command_suggestion(app: &mut App) -> bool {
    if app.command_suggestions.is_empty() {
        return false;
    }
    let idx = app
        .command_select
        .selected
        .min(app.command_suggestions.len().saturating_sub(1));
    let suggestion = app.command_suggestions[idx].clone();
    let (row, _) = app.input.cursor();
    let mut lines = app.input.lines().to_vec();
    if row >= lines.len() {
        return false;
    }
    let line = lines[row].clone();
    if !line.starts_with('/') {
        return false;
    }
    let cmd_end_char = find_first_whitespace(&line).unwrap_or(line.chars().count());
    let cmd_end_byte = byte_index_from_char(&line, cmd_end_char);
    let mut new_line = match suggestion.kind {
        CommandSuggestionKind::Command => {
            let rest = &line[cmd_end_byte..];
            let mut updated = format!("{}{}", suggestion.insert, rest);
            if rest.is_empty() && command_has_args(&suggestion.insert) {
                updated.push(' ');
            }
            updated
        }
        CommandSuggestionKind::Argument => {
            let cmd = line[..cmd_end_byte].trim_end();
            format!("{cmd} {}", suggestion.insert)
        }
    };
    if new_line.trim().is_empty() {
        return false;
    }
    let mut new_col = new_line.chars().count();
    if let CommandSuggestionKind::Command = suggestion.kind {
        if command_has_args(&suggestion.insert) && !new_line.ends_with(' ') {
            new_line.push(' ');
            new_col += 1;
        }
    }
    lines[row] = new_line;
    app.input = tui_textarea::TextArea::from(lines);
    app.input
        .move_cursor(tui_textarea::CursorMove::Jump(row as u16, new_col as u16));
    true
}

pub(crate) fn draw_command_suggestions(
    f: &mut ratatui::Frame<'_>,
    msg_area: Rect,
    input_area: Rect,
    app: &mut App,
    theme: &RenderTheme,
) {
    if app.command_suggestions.is_empty() {
        return;
    }
    if app.focus != Focus::Input {
        return;
    }
    let area = command_suggestions_area(msg_area, input_area, app.command_suggestions.len());
    let viewport = visible_rows(area);
    app.command_select
        .clamp_with_viewport(app.command_suggestions.len(), viewport);
    let header = Row::new(vec![Cell::from("候选"), Cell::from("说明")]).style(header_style(theme));
    let rows = app.command_suggestions.iter().map(|item| {
        Row::new(vec![
            Cell::from(item.label.clone()),
            Cell::from(item.description.clone()),
        ])
    });
    let table = OverlayTable {
        title: Line::from("命令补全 · Tab 应用 · ↑↓ 选择"),
        header,
        rows: rows.collect(),
        widths: vec![Constraint::Length(24), Constraint::Min(10)],
        selected: app.command_select.selected,
        scroll: app.command_select.scroll,
        theme,
    };
    draw_overlay_table(f, area, table);
}

pub(crate) fn command_suggestions_area(msg_area: Rect, input_area: Rect, rows: usize) -> Rect {
    let desired = rows.max(1) as u16 + 3;
    let max_height = msg_area.height.max(3);
    let height = desired.min(12).min(max_height).max(3);
    let width = input_area.width.max(10);
    let x = input_area.x;
    let y = input_area.y.saturating_sub(height).max(msg_area.y);
    Rect {
        x,
        y,
        width,
        height,
    }
}

pub(crate) fn command_suggestions_row_at(
    msg_area: Rect,
    input_area: Rect,
    rows: usize,
    scroll: usize,
    mouse_x: u16,
    mouse_y: u16,
) -> Option<usize> {
    let area = command_suggestions_area(msg_area, input_area, rows);
    row_at(area, rows, scroll, mouse_x, mouse_y)
}

pub(crate) fn handle_command_suggestion_click(
    app: &mut App,
    msg_area: Rect,
    input_area: Rect,
    mouse_x: u16,
    mouse_y: u16,
) -> bool {
    if app.command_suggestions.is_empty() {
        return false;
    }
    let rows = app.command_suggestions.len();
    let Some(row) = command_suggestions_row_at(
        msg_area,
        input_area,
        rows,
        app.command_select.scroll,
        mouse_x,
        mouse_y,
    ) else {
        return false;
    };
    app.command_select.select(row);
    if apply_command_suggestion(app) {
        refresh_command_suggestions(app);
    }
    true
}

fn current_input_suggestions(app: &App) -> Vec<CommandSuggestion> {
    let (row, col) = app.input.cursor();
    let lines = app.input.lines();
    if row >= lines.len() {
        return Vec::new();
    }
    let line = &lines[row];
    command_suggestions_for_input(line, col)
}

fn find_first_whitespace(line: &str) -> Option<usize> {
    line.chars().position(|ch| ch.is_whitespace())
}

fn byte_index_from_char(line: &str, char_idx: usize) -> usize {
    line.char_indices()
        .nth(char_idx)
        .map(|(idx, _)| idx)
        .unwrap_or(line.len())
}
