use crate::render::RenderTheme;
use crate::ui::jump::JumpRow;
use crate::ui::overlay_table::{OverlayTable, draw_overlay_table, header_style};
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::shortcut_help::{HelpRow, help_popup_area, help_rows};
use std::error::Error;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};

use super::super::bindings::bind_event;
use super::super::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::lifecycle::{EventResult, Widget};
use super::overlay_table::{OverlayTableController, clamp_overlay_tables};

pub(crate) struct HelpWidget {
    _private: (),
}

impl HelpWidget {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl Widget for HelpWidget {
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
        draw_shortcut_help(
            frame.frame,
            rect,
            frame.view.help.selected,
            frame.view.help.scroll,
            frame.state.theme,
        );
        Ok(())
    }
}

fn draw_shortcut_help(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    selected: usize,
    scroll: usize,
    theme: &RenderTheme,
) {
    let rows = help_rows();
    let popup = help_popup_area(area, rows.len());
    let table = build_help_table(&rows, selected, scroll, theme);
    draw_overlay_table(f, popup, table);
}

fn build_help_table<'a>(
    rows: &'a [HelpRow],
    selected: usize,
    scroll: usize,
    theme: &'a RenderTheme,
) -> OverlayTable<'a> {
    let header = help_table_header(theme);
    let body = help_table_rows(rows);
    OverlayTable {
        title: Line::from("帮助 · F3/Esc 退出"),
        header,
        rows: body,
        widths: help_table_widths(),
        selected,
        scroll,
        theme,
    }
}

fn help_table_header(theme: &RenderTheme) -> Row<'static> {
    Row::new(vec![
        Cell::from("类型"),
        Cell::from("触发"),
        Cell::from("说明"),
    ])
    .style(header_style(theme))
}

fn help_table_rows(rows: &[HelpRow]) -> Vec<Row<'static>> {
    rows.iter()
        .map(|row| {
            Row::new(vec![
                Cell::from(row.kind),
                Cell::from(row.trigger.clone()),
                Cell::from(row.description),
            ])
        })
        .collect()
}

fn help_table_widths() -> Vec<Constraint> {
    vec![
        Constraint::Length(8),
        Constraint::Length(20),
        Constraint::Min(10),
    ]
}
