use super::popup_layout::file_patch_popup_layout;
use crate::render::RenderTheme;
use crate::framework::widget_system::draw::style::{base_fg, base_style};
use super::popup_text::{build_patch_text, patch_max_scroll};
use crate::framework::widget_system::widgets::jump::JumpRow;
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::interaction::selection::{Selection, apply_selection_to_text};
use crate::framework::widget_system::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use std::error::Error;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

use super::buttons::render_buttons;
use super::event::{handle_key_event, handle_mouse_event};
use super::selection::clamp_patch_scroll;

pub(crate) struct FilePatchWidget {
    pub(super) apply_btn: crate::framework::widget_system::widgets::button::ButtonWidget,
    pub(super) cancel_btn: crate::framework::widget_system::widgets::button::ButtonWidget,
}

impl FilePatchWidget {
    pub(crate) fn new() -> Self {
        Self {
            apply_btn: crate::framework::widget_system::widgets::button::ButtonWidget::new("应用修改"),
            cancel_btn: crate::framework::widget_system::widgets::button::ButtonWidget::new("取消"),
        }
    }
}

impl Widget for FilePatchWidget {
    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        match event {
            crossterm::event::Event::Mouse(m) => handle_mouse_event(self, ctx, layout, update, *m),
            crossterm::event::Event::Key(_) => {
                handle_key_event(ctx, layout, update, jump_rows, event)
            }
            _ => Ok(EventResult::ignored()),
        }
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        let hover = render_popup(frame, rect)?;
        render_buttons(self, rect, hover, frame.state.theme, frame, layout, update);
        Ok(())
    }
}

fn render_popup(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    rect: ratatui::layout::Rect,
) -> Result<Option<crate::framework::widget_system::runtime::state::FilePatchHover>, Box<dyn Error>> {
    let active_tab = frame.state.active_tab;
    let Some(tab_state) = frame.state.tabs.get_mut(active_tab) else {
        return Ok(None);
    };
    let Some(pending) = tab_state.app.pending_file_patch.clone() else {
        return Ok(None);
    };
    let popup = file_patch_popup_layout(rect);
    clamp_patch_scroll(frame.state.theme, tab_state, &pending, popup);
    draw_file_patch_popup_base(
        frame.frame,
        rect,
        &pending,
        tab_state.app.file_patch_scroll,
        tab_state.app.file_patch_selection,
        frame.state.theme,
    );
    Ok(tab_state.app.file_patch_hover)
}

fn draw_file_patch_popup_base(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &crate::framework::widget_system::runtime::state::PendingFilePatch,
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
    let mask_x = popup.x.saturating_sub(super::popup_layout::OUTER_MARGIN).max(area.x);
    let mask_y = popup.y.saturating_sub(super::popup_layout::OUTER_MARGIN).max(area.y);
    let mask_w = popup
        .width
        .saturating_add(super::popup_layout::OUTER_MARGIN.saturating_mul(2))
        .min(max_x.saturating_sub(mask_x));
    let mask_h = popup
        .height
        .saturating_add(super::popup_layout::OUTER_MARGIN.saturating_mul(2))
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
    layout: super::popup_layout::FilePatchPopupLayout,
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
    pending: &crate::framework::widget_system::runtime::state::PendingFilePatch,
    layout: super::popup_layout::FilePatchPopupLayout,
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
