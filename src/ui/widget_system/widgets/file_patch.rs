use crate::ui::file_patch_popup::draw_file_patch_popup_base;
use crate::ui::file_patch_popup_layout::{FilePatchPopupLayout, file_patch_popup_layout};
use crate::ui::file_patch_popup_text::patch_max_scroll;
use crate::ui::jump::JumpRow;
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::selection::{Selection, chat_position_from_mouse, extract_selection};
use crate::ui::state::{FilePatchHover, PendingCommand};
use ratatui::style::{Modifier, Style};
use std::error::Error;

use crate::ui::clipboard;

use super::super::bindings::bind_event;
use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::lifecycle::{EventResult, Widget};
use super::button::ButtonWidget;
use super::overlay_table::OverlayTableController;

pub(crate) struct FilePatchWidget {
    apply_btn: ButtonWidget,
    cancel_btn: ButtonWidget,
}

impl FilePatchWidget {
    pub(crate) fn new() -> Self {
        Self {
            apply_btn: ButtonWidget::new("应用修改"),
            cancel_btn: ButtonWidget::new("取消"),
        }
    }
}

impl Widget for FilePatchWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

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
            crossterm::event::Event::Mouse(m) => {
                let active_tab = *ctx.active_tab;
                let Some(pending) = ctx
                    .tabs
                    .get(active_tab)
                    .and_then(|tab| tab.app.pending_file_patch.clone())
                else {
                    return Ok(EventResult::ignored());
                };
                let popup = file_patch_popup_layout(layout.size);
                if is_mouse_drag(m.kind) {
                    if let Some(tab_state) = ctx.tabs.get_mut(active_tab)
                        && handle_file_patch_selection_drag(
                            tab_state,
                            &pending,
                            popup,
                            ctx.theme,
                            *m,
                        )
                    {
                        return Ok(EventResult::handled());
                    }
                }
                if is_mouse_up(m.kind) {
                    if let Some(tab_state) = ctx.tabs.get_mut(active_tab)
                        && clear_file_patch_selection(tab_state)
                    {
                        return Ok(EventResult::handled());
                    }
                }
                if is_mouse_moved(m.kind) {
                    if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
                        tab_state.app.file_patch_hover = hover_at(*m, popup);
                    }
                    return Ok(EventResult::handled());
                }
                if let Some(delta) = scroll_delta(m.kind) {
                    if let Some(tab_state) = ctx.tabs.get_mut(active_tab)
                        && handle_file_patch_scroll(
                            *m,
                            ctx.theme,
                            tab_state,
                            &pending,
                            popup,
                            delta,
                        )
                    {
                        return Ok(EventResult::handled());
                    }
                }
                if is_mouse_down(m.kind) {
                    if let Some(tab_state) = ctx.tabs.get_mut(active_tab)
                        && handle_file_patch_selection_start(
                            tab_state,
                            &pending,
                            popup,
                            ctx.theme,
                            *m,
                        )
                    {
                        return Ok(EventResult::handled());
                    }
                    if handle_file_patch_buttons(
                        self,
                        ctx,
                        active_tab,
                        *m,
                        popup,
                        ctx.theme,
                        layout,
                        update,
                    ) {
                        return Ok(EventResult::handled());
                    }
                    if !point_in_rect(m.column, m.row, layout.layout.tabs_area)
                        && !point_in_rect(m.column, m.row, layout.layout.category_area)
                    {
                        ctx.view.overlay.close();
                        if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
                            tab_state.app.file_patch_hover = None;
                        }
                        return Ok(EventResult::handled());
                    }
                }
                Ok(EventResult::ignored())
            }
            crossterm::event::Event::Key(_) => {
                if let crossterm::event::Event::Key(key) = event
                    && is_ctrl_c(*key)
                {
                    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab)
                        && let Some(pending) = tab_state.app.pending_file_patch.clone()
                        && copy_file_patch_selection(tab_state, &pending, layout, ctx.theme)
                    {
                        return Ok(EventResult::ignored());
                    }
                }
                let mut binding = bind_event(ctx, layout, update);
                let mut controller = OverlayTableController {
                    dispatch: binding.dispatch,
                    layout: binding.layout,
                    view: binding.view,
                    jump_rows,
                };
                controller.handle_event(event)
            }
            _ => Ok(EventResult::ignored()),
        }
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        let active_tab = frame.state.active_tab;
        let hover = {
            let Some(tab_state) = frame.state.tabs.get_mut(active_tab) else {
                return Ok(());
            };
            let Some(pending) = tab_state.app.pending_file_patch.clone() else {
                return Ok(());
            };
            let popup = file_patch_popup_layout(frame.frame.area());
            clamp_patch_scroll(frame.state.theme, tab_state, &pending, popup);
            draw_file_patch_popup_base(
                frame.frame,
                frame.frame.area(),
                &pending,
                tab_state.app.file_patch_scroll,
                tab_state.app.file_patch_selection,
                frame.state.theme,
            );
            tab_state.app.file_patch_hover
        };
        render_buttons(
            self,
            frame.frame.area(),
            hover,
            frame.state.theme,
            frame,
            layout,
            update,
        );
        Ok(())
    }
}

fn render_buttons(
    widget: &mut FilePatchWidget,
    area: ratatui::layout::Rect,
    hover: Option<FilePatchHover>,
    theme: &crate::render::RenderTheme,
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
) {
    let popup = file_patch_popup_layout(area);
    widget.apply_btn.set_rect(popup.apply_btn);
    widget.cancel_btn.set_rect(popup.cancel_btn);
    widget.apply_btn.set_visible(true);
    widget.cancel_btn.set_visible(true);
    widget.apply_btn.set_bordered(true);
    widget.cancel_btn.set_bordered(true);
    widget
        .apply_btn
        .set_style(button_style(hover, FilePatchHover::Apply, theme));
    widget
        .cancel_btn
        .set_style(button_style(hover, FilePatchHover::Cancel, theme));
    let _ = widget
        .apply_btn
        .render(frame, layout, update, popup.apply_btn);
    let _ = widget
        .cancel_btn
        .render(frame, layout, update, popup.cancel_btn);
}

fn handle_file_patch_buttons(
    widget: &mut FilePatchWidget,
    ctx: &mut EventCtx<'_>,
    active_tab: usize,
    m: crossterm::event::MouseEvent,
    popup: FilePatchPopupLayout,
    theme: &crate::render::RenderTheme,
    layout: &FrameLayout,
    update: &UpdateOutput,
) -> bool {
    if !point_in_rect(m.column, m.row, popup.popup) {
        return false;
    }
    let hover = ctx
        .tabs
        .get(active_tab)
        .map(|tab| tab.app.file_patch_hover)
        .unwrap_or(None);
    widget.apply_btn.set_rect(popup.apply_btn);
    widget.cancel_btn.set_rect(popup.cancel_btn);
    widget.apply_btn.set_visible(true);
    widget.cancel_btn.set_visible(true);
    widget.apply_btn.set_bordered(true);
    widget.cancel_btn.set_bordered(true);
    widget
        .apply_btn
        .set_style(button_style(hover, FilePatchHover::Apply, theme));
    widget
        .cancel_btn
        .set_style(button_style(hover, FilePatchHover::Cancel, theme));
    let event = crossterm::event::Event::Mouse(m);
    if widget
        .apply_btn
        .event(ctx, &event, layout, update, &[], popup.apply_btn)
        .map(|r| r.handled)
        .unwrap_or(false)
    {
        if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
            tab_state.app.pending_command = Some(PendingCommand::ApplyFilePatch);
            tab_state.app.file_patch_hover = None;
            ctx.view.overlay.close();
            return true;
        }
    }
    if widget
        .cancel_btn
        .event(ctx, &event, layout, update, &[], popup.cancel_btn)
        .map(|r| r.handled)
        .unwrap_or(false)
    {
        if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
            tab_state.app.pending_command = Some(PendingCommand::CancelFilePatch);
            tab_state.app.file_patch_hover = None;
            ctx.view.overlay.close();
            return true;
        }
    }
    false
}

fn handle_file_patch_scroll(
    m: crossterm::event::MouseEvent,
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingFilePatch,
    popup: FilePatchPopupLayout,
    delta: i32,
) -> bool {
    if !point_in_rect(m.column, m.row, popup.popup) {
        return false;
    }
    let max_scroll = patch_max_scroll(
        &pending.preview,
        popup.preview_area.width,
        popup.preview_area.height,
        theme,
    );
    apply_scroll(&mut tab_state.app.file_patch_scroll, delta, max_scroll);
    true
}

fn handle_file_patch_selection_start(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingFilePatch,
    popup: FilePatchPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) -> bool {
    if !point_in_rect(m.column, m.row, popup.preview_area) {
        return false;
    }
    let (text, _) = crate::ui::file_patch_popup_text::build_patch_text(
        &pending.preview,
        popup.preview_area.width,
        popup.preview_area.height,
        tab_state.app.file_patch_scroll,
        theme,
    );
    let pos = selection_position_for_panel(
        &text,
        tab_state.app.file_patch_scroll,
        popup.preview_area,
        m,
    );
    tab_state.app.file_patch_selecting = true;
    tab_state.app.file_patch_selection = Some(Selection { start: pos, end: pos });
    true
}

fn handle_file_patch_selection_drag(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingFilePatch,
    popup: FilePatchPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) -> bool {
    if !tab_state.app.file_patch_selecting {
        return false;
    }
    let (text, _) = crate::ui::file_patch_popup_text::build_patch_text(
        &pending.preview,
        popup.preview_area.width,
        popup.preview_area.height,
        tab_state.app.file_patch_scroll,
        theme,
    );
    let pos = selection_position_for_panel(
        &text,
        tab_state.app.file_patch_scroll,
        popup.preview_area,
        m,
    );
    let next = match tab_state.app.file_patch_selection {
        Some(existing) => Selection {
            start: existing.start,
            end: pos,
        },
        None => Selection { start: pos, end: pos },
    };
    tab_state.app.file_patch_selection = Some(next);
    true
}

fn clear_file_patch_selection(tab_state: &mut crate::ui::runtime_helpers::TabState) -> bool {
    if !tab_state.app.file_patch_selecting {
        return false;
    }
    tab_state.app.file_patch_selecting = false;
    if tab_state
        .app
        .file_patch_selection
        .map(|sel| sel.is_empty())
        .unwrap_or(false)
    {
        tab_state.app.file_patch_selection = None;
    }
    true
}

fn selection_position_for_panel(
    text: &ratatui::text::Text<'static>,
    scroll: usize,
    area: ratatui::layout::Rect,
    m: crossterm::event::MouseEvent,
) -> (usize, usize) {
    let scroll_u16 = scroll.min(u16::MAX as usize) as u16;
    chat_position_from_mouse(text, scroll_u16, area, m.column, m.row)
}

fn hover_at(m: crossterm::event::MouseEvent, popup: FilePatchPopupLayout) -> Option<FilePatchHover> {
    if point_in_rect(m.column, m.row, popup.apply_btn) {
        Some(FilePatchHover::Apply)
    } else if point_in_rect(m.column, m.row, popup.cancel_btn) {
        Some(FilePatchHover::Cancel)
    } else {
        None
    }
}

fn clamp_patch_scroll(
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingFilePatch,
    layout: FilePatchPopupLayout,
) {
    let max_scroll = patch_max_scroll(
        &pending.preview,
        layout.preview_area.width,
        layout.preview_area.height,
        theme,
    );
    if tab_state.app.file_patch_scroll > max_scroll {
        tab_state.app.file_patch_scroll = max_scroll;
    }
}

fn button_style(
    hover: Option<FilePatchHover>,
    target: FilePatchHover,
    theme: &crate::render::RenderTheme,
) -> Style {
    match hover {
        Some(h) if h == target => Style::default()
            .bg(crate::ui::draw::style::selection_bg(theme.bg))
            .fg(crate::ui::draw::style::base_fg(theme))
            .add_modifier(Modifier::BOLD),
        _ => crate::ui::draw::style::base_style(theme),
    }
}

fn point_in_rect(x: u16, y: u16, rect: ratatui::layout::Rect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

fn is_mouse_down(kind: crossterm::event::MouseEventKind) -> bool {
    matches!(kind, crossterm::event::MouseEventKind::Down(_))
}

fn is_mouse_moved(kind: crossterm::event::MouseEventKind) -> bool {
    matches!(kind, crossterm::event::MouseEventKind::Moved)
}

fn is_mouse_up(kind: crossterm::event::MouseEventKind) -> bool {
    matches!(kind, crossterm::event::MouseEventKind::Up(_))
}

fn is_mouse_drag(kind: crossterm::event::MouseEventKind) -> bool {
    matches!(kind, crossterm::event::MouseEventKind::Drag(_))
}

fn is_ctrl_c(key: crossterm::event::KeyEvent) -> bool {
    key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
        && key.code == crossterm::event::KeyCode::Char('c')
}

fn scroll_delta(kind: crossterm::event::MouseEventKind) -> Option<i32> {
    match kind {
        crossterm::event::MouseEventKind::ScrollUp => Some(-crate::ui::scroll::SCROLL_STEP_I32),
        crossterm::event::MouseEventKind::ScrollDown => Some(crate::ui::scroll::SCROLL_STEP_I32),
        _ => None,
    }
}

fn apply_scroll(current: &mut usize, delta: i32, max: usize) {
    let next = (i32::try_from(*current).unwrap_or(0) + delta).max(0) as usize;
    *current = next.min(max);
}

fn copy_file_patch_selection(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingFilePatch,
    layout: &FrameLayout,
    theme: &crate::render::RenderTheme,
) -> bool {
    let Some(selection) = tab_state.app.file_patch_selection else {
        return false;
    };
    let popup = file_patch_popup_layout(layout.size);
    let lines = crate::ui::file_patch_popup_text::patch_plain_lines(
        &pending.preview,
        popup.preview_area.width,
        theme,
    );
    let text = extract_selection(&lines, selection);
    if !text.is_empty() {
        clipboard::set(&text);
    }
    true
}
