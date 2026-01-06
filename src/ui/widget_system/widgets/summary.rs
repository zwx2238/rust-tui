use crate::ui::jump::JumpRow;
use crate::ui::runtime_loop_steps::FrameLayout;
use std::error::Error;

use super::super::bindings::bind_event;
use super::super::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::lifecycle::{EventResult, Widget};
use super::overlay_table::{OverlayTableController, clamp_overlay_tables};

pub(crate) struct SummaryWidget {
    _private: (),
}

impl SummaryWidget {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl Widget for SummaryWidget {
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
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        clamp_overlay_tables(frame.view, frame.state, frame.jump_rows.len());
        let layout = summary_layout(frame);
        let rows = summary_rows(frame, &layout);
        draw_summary_layout(frame, &layout);
        draw_summary_table(frame, &layout, &rows);
        update_summary_order(frame, &rows);
        Ok(())
    }
}

fn summary_layout(
    frame: &WidgetFrame<'_, '_, '_, '_>,
) -> crate::ui::summary::layout::SummaryLayout {
    let size = ratatui::layout::Size {
        width: frame.frame.area().width,
        height: frame.frame.area().height,
    };
    crate::ui::summary::layout::build_summary_layout(size, frame.state.categories)
}

fn summary_rows(
    frame: &WidgetFrame<'_, '_, '_, '_>,
    layout: &crate::ui::summary::layout::SummaryLayout,
) -> Vec<crate::ui::summary::SummaryRow> {
    let mut rows =
        crate::ui::summary::build_summary_rows(frame.state.tabs(), layout.max_latest_width.max(10));
    crate::ui::summary::sort_summary_rows(&mut rows, frame.view.summary_sort);
    rows
}

fn draw_summary_layout(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    layout: &crate::ui::summary::layout::SummaryLayout,
) {
    crate::ui::summary::layout::draw_summary_layout(
        crate::ui::summary::layout::DrawSummaryLayoutParams {
            f: frame.frame,
            theme: frame.state.theme,
            tab_labels: frame.state.tab_labels,
            active_tab_pos: frame.state.active_tab_pos,
            categories: frame.state.categories,
            active_category: frame.state.active_category,
            header_note: frame.state.header_note,
            startup_text: frame.state.startup_text,
            header_area: layout.header_area,
            category_area: layout.category_area,
            tabs_area: layout.tabs_area,
            footer_area: layout.footer_area,
        },
    );
}

fn draw_summary_table(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    layout: &crate::ui::summary::layout::SummaryLayout,
    rows: &[crate::ui::summary::SummaryRow],
) {
    crate::ui::summary::table::draw_summary_table(
        frame.frame,
        layout.body_area,
        rows,
        frame.view.summary.selected,
        frame.view.summary.scroll,
        frame.state.theme,
        frame.view.summary_sort,
    );
}

fn update_summary_order(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    rows: &[crate::ui::summary::SummaryRow],
) {
    frame.view.summary_order = rows.iter().map(|r| r.tab_index).collect();
}
