use crate::llm::prompts::SystemPrompt;
use crate::render::RenderTheme;
use crate::ui::overlay_table::{OverlayTable, centered_area, draw_overlay_table, header_style};
use crate::ui::text_utils::{collapse_text, truncate_to_width};
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};
use unicode_width::UnicodeWidthStr;

const POPUP_MAX_HEIGHT: u16 = 18;

pub fn draw_prompt_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    prompts: &[SystemPrompt],
    selected: usize,
    scroll: usize,
    theme: &RenderTheme,
) {
    let popup = prompt_popup_area(area, prompts.len());
    let popup_spec = build_prompt_table(prompts, selected, scroll, theme, popup);
    draw_overlay_table(f, popup, popup_spec);
}

fn build_prompt_table<'a>(
    prompts: &[SystemPrompt],
    selected: usize,
    scroll: usize,
    theme: &'a RenderTheme,
    popup: Rect,
) -> OverlayTable<'a> {
    let role_width = role_col_width(popup, prompts);
    let header =
        Row::new(vec![Cell::from("角色"), Cell::from("系统提示词")]).style(header_style(theme));
    let body = prompts.iter().map(|p| {
        Row::new(vec![
            Cell::from(p.key.clone()),
            Cell::from(truncate_to_width(
                &collapse_text(&p.content),
                max_preview_width(popup, role_width),
            )),
        ])
    });
    OverlayTable {
        title: Line::from("系统提示词 · Enter 确认 · Esc 取消"),
        header,
        rows: body.collect(),
        widths: vec![Constraint::Length(role_width), Constraint::Min(10)],
        selected,
        scroll,
        theme,
    }
}

pub fn prompt_popup_area(area: Rect, rows: usize) -> Rect {
    centered_area(area, 80, rows, POPUP_MAX_HEIGHT)
}

fn max_preview_width(area: Rect, role_width: u16) -> usize {
    area.width.saturating_sub(role_width).saturating_sub(4) as usize
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

// layout helpers are centralized in overlay_table

// selection color handled by overlay_table

// text utilities are centralized in text_utils
