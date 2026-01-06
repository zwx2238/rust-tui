use crate::render::RenderTheme;
use crate::ui::draw::style::{base_fg, base_style};
use crate::ui::file_patch_popup_layout::{
    FilePatchPopupLayout, OUTER_MARGIN, file_patch_popup_layout,
};
use crate::ui::file_patch_popup_text::{build_patch_text, patch_max_scroll};
use crate::ui::selection::{Selection, apply_selection_to_text};
use crate::ui::state::PendingFilePatch;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

pub(crate) fn draw_file_patch_popup_base(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingFilePatch,
    scroll: usize,
    selection: Option<Selection>,
    theme: &RenderTheme,
) {
    let layout = file_patch_popup_layout(area);
    let mask = popup_mask(area, layout.popup);
    render_mask(f, theme, mask);
    render_popup_base(f, theme, layout.popup);
    let (preview_text, total_lines) = build_patch_text(
        &pending.preview,
        layout.preview_area.width,
        layout.preview_area.height,
        scroll,
        theme,
    );
    render_preview_panel(f, theme, layout, preview_text, selection, scroll);
    render_preview_scrollbar(f, theme, pending, layout, total_lines, scroll);
}

fn popup_mask(area: Rect, popup: Rect) -> Rect {
    let max_x = area.x.saturating_add(area.width);
    let max_y = area.y.saturating_add(area.height);
    let mask_x = popup.x.saturating_sub(OUTER_MARGIN).max(area.x);
    let mask_y = popup.y.saturating_sub(OUTER_MARGIN).max(area.y);
    let mask_w = popup
        .width
        .saturating_add(OUTER_MARGIN.saturating_mul(2))
        .min(max_x.saturating_sub(mask_x));
    let mask_h = popup
        .height
        .saturating_add(OUTER_MARGIN.saturating_mul(2))
        .min(max_y.saturating_sub(mask_y));
    Rect {
        x: mask_x,
        y: mask_y,
        width: mask_w,
        height: mask_h,
    }
}

fn render_mask(f: &mut ratatui::Frame<'_>, theme: &RenderTheme, mask: Rect) {
    f.render_widget(Clear, mask);
    let mask_block = Block::default().style(base_style(theme));
    f.render_widget(mask_block, mask);
}

fn render_popup_base(f: &mut ratatui::Frame<'_>, theme: &RenderTheme, popup: Rect) {
    f.render_widget(Clear, popup);
    let title = "文件修改预览 · 仅预览，需确认后应用";
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(vec![Span::styled(
            title,
            Style::default()
                .fg(base_fg(theme))
                .add_modifier(Modifier::BOLD),
        )]))
        .style(base_style(theme))
        .border_style(Style::default().fg(base_fg(theme)));
    f.render_widget(block, popup);
}

fn render_preview_panel(
    f: &mut ratatui::Frame<'_>,
    theme: &RenderTheme,
    layout: FilePatchPopupLayout,
    preview_text: Text<'static>,
    selection: Option<Selection>,
    scroll: usize,
) {
    let preview_text = apply_selection_if_needed(preview_text, scroll, selection);
    let preview_para = Paragraph::new(preview_text)
        .style(base_style(theme))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(preview_para, layout.preview_area);
}

fn render_preview_scrollbar(
    f: &mut ratatui::Frame<'_>,
    theme: &RenderTheme,
    pending: &PendingFilePatch,
    layout: FilePatchPopupLayout,
    total_lines: usize,
    scroll: usize,
) {
    if total_lines <= layout.preview_area.height as usize {
        return;
    }
    let viewport_len = layout.preview_area.height as usize;
    let max_scroll = patch_max_scroll(
        &pending.preview,
        layout.preview_area.width,
        layout.preview_area.height,
        theme,
    );
    let mut state = ScrollbarState::new(max_scroll.saturating_add(1))
        .position(scroll.min(max_scroll))
        .viewport_content_length(viewport_len);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(Style::default().fg(base_fg(theme)))
        .track_style(Style::default().fg(base_fg(theme)));
    f.render_stateful_widget(scrollbar, layout.preview_scrollbar_area, &mut state);
}

fn apply_selection_if_needed(
    text: Text<'static>,
    scroll: usize,
    selection: Option<Selection>,
) -> Text<'static> {
    let Some(selection) = selection else {
        return text;
    };
    if selection.is_empty() {
        return text;
    }
    apply_selection_to_text(
        &text,
        scroll,
        selection,
        Style::default().bg(Color::DarkGray),
    )
}
