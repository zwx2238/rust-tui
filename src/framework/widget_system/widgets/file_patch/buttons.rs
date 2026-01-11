use super::popup_layout::{FilePatchPopupLayout, file_patch_popup_layout};
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::runtime::state::{FilePatchHover, PendingCommand};
use crate::framework::widget_system::context::{EventCtx, UpdateOutput, WidgetFrame};
use crate::framework::widget_system::lifecycle::Widget;
use ratatui::style::{Modifier, Style};

use super::super::button::ButtonWidget;
use super::helpers::point_in_rect;
use super::widget::FilePatchWidget;

pub(super) struct FilePatchButtonParams<'a> {
    pub(super) active_tab: usize,
    pub(super) popup: FilePatchPopupLayout,
    pub(super) theme: &'a crate::render::RenderTheme,
    pub(super) layout: &'a FrameLayout,
    pub(super) update: &'a UpdateOutput,
}

pub(super) fn render_buttons(
    widget: &mut FilePatchWidget,
    area: ratatui::layout::Rect,
    hover: Option<FilePatchHover>,
    theme: &crate::render::RenderTheme,
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
) {
    let popup = file_patch_popup_layout(area);
    configure_buttons(widget, popup, hover, theme);
    let _ = widget
        .apply_btn
        .render(frame, layout, update, popup.apply_btn);
    let _ = widget
        .cancel_btn
        .render(frame, layout, update, popup.cancel_btn);
}

pub(super) fn handle_file_patch_buttons(
    widget: &mut FilePatchWidget,
    ctx: &mut EventCtx<'_>,
    m: crossterm::event::MouseEvent,
    params: &FilePatchButtonParams<'_>,
) -> bool {
    if !point_in_rect(m.column, m.row, params.popup.popup) {
        return false;
    }
    let hover = ctx
        .tabs
        .get(params.active_tab)
        .map(|tab| tab.app.file_patch_hover)
        .unwrap_or(None);
    configure_buttons(widget, params.popup, hover, params.theme);
    let event = crossterm::event::Event::Mouse(m);
    if button_clicked(
        &mut widget.apply_btn,
        ctx,
        params.layout,
        params.update,
        params.popup.apply_btn,
        &event,
    ) {
        return apply_command(ctx, params.active_tab, PendingCommand::ApplyFilePatch);
    }
    if button_clicked(
        &mut widget.cancel_btn,
        ctx,
        params.layout,
        params.update,
        params.popup.cancel_btn,
        &event,
    ) {
        return apply_command(ctx, params.active_tab, PendingCommand::CancelFilePatch);
    }
    false
}

fn configure_buttons(
    widget: &mut FilePatchWidget,
    popup: FilePatchPopupLayout,
    hover: Option<FilePatchHover>,
    theme: &crate::render::RenderTheme,
) {
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
}

fn button_clicked(
    button: &mut ButtonWidget,
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    rect: ratatui::layout::Rect,
    event: &crossterm::event::Event,
) -> bool {
    button
        .event(ctx, event, layout, update, rect)
        .map(|r| r.handled)
        .unwrap_or(false)
}

fn apply_command(ctx: &mut EventCtx<'_>, active_tab: usize, command: PendingCommand) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(active_tab) {
        tab_state.app.pending_command = Some(command);
        tab_state.app.file_patch_hover = None;
        ctx.view.overlay.close();
        return true;
    }
    false
}

fn button_style(
    hover: Option<FilePatchHover>,
    target: FilePatchHover,
    theme: &crate::render::RenderTheme,
) -> Style {
    match hover {
        Some(h) if h == target => Style::default()
            .bg(crate::framework::widget_system::draw::style::selection_bg(theme.bg))
            .fg(crate::framework::widget_system::draw::style::base_fg(theme))
            .add_modifier(Modifier::BOLD),
        _ => crate::framework::widget_system::draw::style::base_style(theme),
    }
}
