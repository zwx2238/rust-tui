use crate::model_registry::ModelProfile;
use crate::render::RenderTheme;
use crate::ui::jump::JumpRow;
use crate::ui::overlay_table::{OverlayTable, draw_overlay_table, header_style};
use crate::ui::runtime_loop_steps::FrameLayout;
use std::error::Error;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};

use super::super::bindings::bind_event;
use super::super::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::lifecycle::{EventResult, Widget};
use super::overlay_table::{OverlayTableController, clamp_overlay_tables};

pub(crate) struct ModelWidget {
    _private: (),
}

impl ModelWidget {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl Widget for ModelWidget {
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
        draw_model_popup(
            frame.frame,
            rect,
            frame.state.models,
            frame.view.model.selected,
            frame.view.model.scroll,
            frame.state.theme,
        );
        Ok(())
    }
}

fn draw_model_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    models: &[ModelProfile],
    selected: usize,
    scroll: usize,
    theme: &RenderTheme,
) {
    let popup = crate::ui::model_popup::model_popup_area(area, models.len());
    let popup_spec = build_model_table(models, selected, scroll, theme);
    draw_overlay_table(f, popup, popup_spec);
}

fn build_model_table<'a>(
    models: &'a [ModelProfile],
    selected: usize,
    scroll: usize,
    theme: &'a RenderTheme,
) -> OverlayTable<'a> {
    OverlayTable {
        title: Line::from(model_title()),
        header: model_header(theme),
        rows: model_body(models),
        widths: model_widths(),
        selected,
        scroll,
        theme,
    }
}

fn model_header(theme: &RenderTheme) -> Row<'static> {
    Row::new(vec![
        Cell::from("名称"),
        Cell::from("模型"),
        Cell::from("Base URL"),
    ])
    .style(header_style(theme))
}

fn model_body<'a>(models: &'a [ModelProfile]) -> Vec<Row<'a>> {
    models
        .iter()
        .map(|m| {
            Row::new(vec![
                Cell::from(m.key.clone()),
                Cell::from(m.model.clone()),
                Cell::from(m.base_url.clone()),
            ])
        })
        .collect()
}

fn model_widths() -> Vec<Constraint> {
    vec![
        Constraint::Length(12),
        Constraint::Length(22),
        Constraint::Min(10),
    ]
}

fn model_title() -> &'static str {
    "模型切换 · Enter 确认 · Esc 取消 · F10/Shift+F10 快速切换"
}
