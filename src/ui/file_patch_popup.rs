use crate::render::RenderTheme;
use crate::ui::file_patch_popup_layout::{OUTER_MARGIN, file_patch_popup_layout};
use crate::ui::file_patch_popup_text::{build_patch_text, patch_max_scroll};
use crate::ui::draw::style::{base_fg, base_style, selection_bg};
use crate::ui::state::{FilePatchHover, PendingFilePatch};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
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
    let layout = file_patch_popup_layout(area);
    let max_x = area.x.saturating_add(area.width);
    let max_y = area.y.saturating_add(area.height);
    let mask_x = layout.popup.x.saturating_sub(OUTER_MARGIN).max(area.x);
    let mask_y = layout.popup.y.saturating_sub(OUTER_MARGIN).max(area.y);
    let mask_w = layout
        .popup
        .width
        .saturating_add(OUTER_MARGIN.saturating_mul(2))
        .min(max_x.saturating_sub(mask_x));
    let mask_h = layout
        .popup
        .height
        .saturating_add(OUTER_MARGIN.saturating_mul(2))
        .min(max_y.saturating_sub(mask_y));
    let mask = Rect {
        x: mask_x,
        y: mask_y,
        width: mask_w,
        height: mask_h,
    };
    f.render_widget(Clear, mask);
    let mask_block = Block::default().style(base_style(theme));
    f.render_widget(mask_block, mask);
    f.render_widget(Clear, layout.popup);
    let mask = Block::default().style(base_style(theme));
    f.render_widget(mask, layout.popup);
    let title = "文件修改预览 · 仅预览，需确认后应用";
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(vec![
            Span::styled(title, Style::default().fg(base_fg(theme)).add_modifier(Modifier::BOLD)),
        ]))
        .style(base_style(theme))
        .border_style(Style::default().fg(base_fg(theme)));
    f.render_widget(block, layout.popup);

    let (preview_text, total_lines) = build_patch_text(
        &pending.preview,
        layout.preview_area.width,
        layout.preview_area.height,
        scroll,
        theme,
    );
    let preview_para = Paragraph::new(preview_text)
        .style(base_style(theme))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(preview_para, layout.preview_area);

    if total_lines > layout.preview_area.height as usize {
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

    let button_style = |target: FilePatchHover| match hover {
        Some(h) if h == target => {
            Style::default().bg(selection_bg(theme.bg)).fg(base_fg(theme)).add_modifier(Modifier::BOLD)
        }
        _ => base_style(theme),
    };
    let apply_style = button_style(FilePatchHover::Apply);
    let cancel_style = button_style(FilePatchHover::Cancel);
    let apply_block = Block::default().borders(Borders::ALL).style(apply_style);
    let cancel_block = Block::default().borders(Borders::ALL).style(cancel_style);
    f.render_widget(apply_block, layout.apply_btn);
    f.render_widget(
        Paragraph::new(Line::from("应用修改"))
            .style(apply_style)
            .alignment(ratatui::layout::Alignment::Center),
        layout.apply_btn,
    );
    f.render_widget(cancel_block, layout.cancel_btn);
    f.render_widget(
        Paragraph::new(Line::from("取消"))
            .style(cancel_style)
            .alignment(ratatui::layout::Alignment::Center),
        layout.cancel_btn,
    );
}
