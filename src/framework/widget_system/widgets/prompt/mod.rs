mod popup;

pub(crate) use popup::prompt_popup_area;

use crate::llm::prompts::SystemPrompt;
use crate::render::RenderTheme;
use crate::framework::widget_system::widgets::jump::JumpRow;
use crate::framework::widget_system::widgets::overlay_table::{OverlayTable, draw_overlay_table, header_style};
use crate::framework::widget_system::interaction::text_utils::{collapse_text, truncate_to_width};
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use std::error::Error;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};
use unicode_width::UnicodeWidthStr;

use super::super::bindings::bind_event;
use super::super::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::lifecycle::{EventResult, Widget};
use super::overlay_table::{OverlayTableController, clamp_overlay_tables};

pub(crate) struct PromptWidget {
    _private: (),
}

impl PromptWidget {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl Widget for PromptWidget {
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
        let binding = bind_event(ctx, layout, update);
        let mut controller = OverlayTableController {
            dispatch: binding.dispatch,
            layout: binding.layout,
            view: binding.view,
            jump_rows,
        };
        controller.handle_event(event)
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        clamp_overlay_tables(frame.view, frame.state, frame.jump_rows.len());
        draw_prompt_popup(
            frame.frame,
            rect,
            frame.state.prompts,
            frame.view.prompt.selected,
            frame.view.prompt.scroll,
            frame.state.theme,
        );
        Ok(())
    }
}

fn draw_prompt_popup(
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
