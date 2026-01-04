use crate::render::RenderTheme;
use crate::ui::draw::style::{base_fg, base_style, selection_bg};
use crate::ui::file_patch_popup_layout::{
    FilePatchPopupLayout, OUTER_MARGIN, file_patch_popup_layout,
};
use crate::ui::file_patch_popup_text::{build_patch_text, patch_max_scroll};
use crate::ui::state::{FilePatchHover, PendingFilePatch};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

pub(crate) fn draw_file_patch_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingFilePatch,
    scroll: usize,
    hover: Option<FilePatchHover>,
    theme: &RenderTheme,
) {
    draw_file_patch_popup_base(f, area, pending, scroll, theme);
    let layout = file_patch_popup_layout(area);
    render_buttons(f, theme, layout, hover);
}

pub(crate) fn draw_file_patch_popup_base(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingFilePatch,
    scroll: usize,
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
    render_preview_panel(f, theme, layout, preview_text);
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
    preview_text: Text<'_>,
) {
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

fn render_buttons(
    f: &mut ratatui::Frame<'_>,
    theme: &RenderTheme,
    layout: FilePatchPopupLayout,
    hover: Option<FilePatchHover>,
) {
    let apply_style = button_style(hover, FilePatchHover::Apply, theme);
    let cancel_style = button_style(hover, FilePatchHover::Cancel, theme);
    render_button(f, layout.apply_btn, "应用修改", apply_style);
    render_button(f, layout.cancel_btn, "取消", cancel_style);
}

fn render_button(f: &mut ratatui::Frame<'_>, area: Rect, label: &str, style: Style) {
    let block = Block::default().borders(Borders::ALL).style(style);
    f.render_widget(block, area);
    f.render_widget(
        Paragraph::new(Line::from(label))
            .style(style)
            .alignment(ratatui::layout::Alignment::Center),
        area,
    );
}

fn button_style(
    hover: Option<FilePatchHover>,
    target: FilePatchHover,
    theme: &RenderTheme,
) -> Style {
    match hover {
        Some(h) if h == target => Style::default()
            .bg(selection_bg(theme.bg))
            .fg(base_fg(theme))
            .add_modifier(Modifier::BOLD),
        _ => base_style(theme),
    }
}
