use crate::render::RenderTheme;
use crate::system_prompts::SystemPrompt;
use crate::ui::popup_table::{draw_table_popup, popup_row_at, popup_visible_rows, TablePopup};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn draw_prompt_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    prompts: &[SystemPrompt],
    selected: usize,
    scroll: usize,
    theme: &RenderTheme,
) {
    let popup = prompt_popup_area(area, prompts.len());
    let role_width = role_col_width(popup, prompts);
    let header = Row::new(vec![Cell::from("角色"), Cell::from("系统提示词")]).style(
        Style::default()
            .fg(theme.fg.unwrap_or(Color::White))
            .add_modifier(Modifier::BOLD),
    );
    let body = prompts.iter().map(|p| {
        Row::new(vec![
            Cell::from(p.key.clone()),
            Cell::from(truncate_to_width(
                &collapse_text(&p.content),
                max_preview_width(popup, role_width),
            )),
        ])
    });
    let popup_spec = TablePopup {
        title: Line::from("系统提示词 · Enter 确认 · Esc 取消"),
        header,
        rows: body.collect(),
        widths: vec![Constraint::Length(role_width), Constraint::Min(10)],
        selected,
        scroll,
        theme,
    };
    draw_table_popup(f, popup, popup_spec);
}

pub fn prompt_popup_area(area: Rect, rows: usize) -> Rect {
    centered_rect(area, 80, popup_height(rows))
}

pub fn prompt_row_at(
    area: Rect,
    rows: usize,
    scroll: usize,
    mouse_x: u16,
    mouse_y: u16,
) -> Option<usize> {
    let popup = prompt_popup_area(area, rows);
    popup_row_at(popup, rows, scroll, mouse_x, mouse_y)
}

pub fn prompt_visible_rows(area: Rect, rows: usize) -> usize {
    let popup = prompt_popup_area(area, rows);
    popup_visible_rows(popup)
}

fn max_preview_width(area: Rect, role_width: u16) -> usize {
    area.width
        .saturating_sub(role_width)
        .saturating_sub(4) as usize
}

fn role_col_width(area: Rect, prompts: &[SystemPrompt]) -> u16 {
    let mut max = "角色".width();
    for p in prompts {
        max = max.max(p.key.width());
    }
    let needed = (max + 2) as u16;
    let max_allowed = area.width.saturating_sub(10).max(8);
    needed.min(max_allowed)
}

fn popup_height(rows: usize) -> u16 {
    let body = rows.max(1) as u16;
    (body + 3).min(18)
}

fn centered_rect(area: Rect, percent_x: u16, height: u16) -> Rect {
    let width = area.width * percent_x / 100;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let h = height.min(area.height.saturating_sub(2)).max(3);
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect { x, y, width, height: h }
}

// selection color handled by popup_table

fn collapse_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_to_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if text.width() <= max_width {
        return text.to_string();
    }
    let ellipsis = "...";
    let mut out = String::new();
    let mut width = 0usize;
    let limit = max_width.saturating_sub(ellipsis.width());
    for ch in text.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(1);
        if width.saturating_add(w) > limit {
            break;
        }
        out.push(ch);
        width = width.saturating_add(w);
    }
    out.push_str(ellipsis);
    out
}
