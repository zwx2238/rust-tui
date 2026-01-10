use crate::ui::jump::JumpRow;
use crate::ui::question_review_popup::draw_question_review_popup;
use crate::ui::runtime_loop_steps::FrameLayout;
use std::error::Error;

use super::super::bindings::bind_event;
use super::super::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::lifecycle::{EventResult, Widget};
use super::overlay_table::{OverlayTableController, clamp_overlay_tables};

pub(crate) struct QuestionReviewWidget {
    _private: (),
}

impl QuestionReviewWidget {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl Widget for QuestionReviewWidget {
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
        let Some(app) = frame.state.active_app() else {
            return Ok(());
        };
        let Some(pending) = app.pending_question_review.as_ref() else {
            return Ok(());
        };
        draw_question_review_popup(
            frame.frame,
            rect,
            pending,
            frame.view.question_review.selected,
            frame.view.question_review.scroll,
            &mut frame.view.question_review_detail_scroll,
            frame.state.theme,
        );
        Ok(())
    }
}
