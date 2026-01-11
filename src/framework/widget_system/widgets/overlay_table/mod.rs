pub(crate) mod state;
mod table;

pub(crate) use state::{OverlayAreas, OverlayRowCounts, with_active_table_handle};
pub(crate) use table::{OverlayTable, centered_area, draw_overlay_table, header_style, row_at, visible_rows};

use crate::framework::widget_system::runtime_dispatch::key_helpers::{
    handle_pre_key_actions, handle_view_action_flow, is_quit_key, resolve_view_action,
};
use crate::framework::widget_system::runtime_dispatch::{
    DispatchContext, LayoutContext, apply_model_selection, apply_prompt_selection,
};
use crate::framework::widget_system::runtime::runtime_view::{ViewAction, ViewState, apply_view_action, handle_view_mouse};
use crate::framework::widget_system::interaction::scroll::SCROLL_STEP_I32;
use crate::framework::widget_system::widgets::help::help_rows_len;
use crate::framework::widget_system::context::RenderState;
use crate::framework::widget_system::lifecycle::EventResult;

pub(crate) struct OverlayTableController<'a> {
    pub(crate) dispatch: DispatchContext<'a>,
    pub(crate) layout: LayoutContext,
    pub(crate) view: &'a mut ViewState,
    pub(crate) jump_rows: &'a [crate::framework::widget_system::widgets::jump::JumpRow],
}

impl<'a> OverlayTableController<'a> {
    pub(crate) fn handle_event(
        &mut self,
        event: &crossterm::event::Event,
    ) -> Result<EventResult, Box<dyn std::error::Error>> {
        match event {
            crossterm::event::Event::Key(key) => self.handle_key(*key),
            crossterm::event::Event::Mouse(m) => {
                self.handle_mouse(*m);
                Ok(EventResult::handled())
            }
            _ => Ok(EventResult::ignored()),
        }
    }

    fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<EventResult, Box<dyn std::error::Error>> {
        if is_quit_key(key) {
            return Ok(EventResult::quit());
        }
        if handle_pre_key_actions(&mut self.dispatch, self.view, key) {
            return Ok(EventResult::handled());
        }
        let action = resolve_view_action(&mut self.dispatch, self.view, key, self.jump_rows);
        if handle_view_action_flow(
            &mut self.dispatch,
            self.layout,
            self.view,
            self.jump_rows,
            action,
            key,
        ) {
            return Ok(EventResult::handled());
        }
        Ok(EventResult::ignored())
    }

    fn handle_mouse(&mut self, m: crossterm::event::MouseEvent) {
        if self.handle_overlay_scroll(m) {
            return;
        }
        let row = self.overlay_row_at(m.column, m.row);
        let action = handle_view_mouse(
            self.view,
            row,
            self.dispatch.tabs.len(),
            self.jump_rows.len(),
            m.kind,
        );
        if let ViewAction::SelectModel(idx) = action {
            apply_model_selection(&mut self.dispatch, idx);
            return;
        }
        if let ViewAction::SelectPrompt(idx) = action {
            apply_prompt_selection(&mut self.dispatch, idx);
            return;
        }
        let _ = apply_view_action(
            action,
            self.jump_rows,
            self.dispatch.tabs,
            self.dispatch.active_tab,
            self.dispatch.categories,
            self.dispatch.active_category,
        );
    }

    fn handle_overlay_scroll(&mut self, m: crossterm::event::MouseEvent) -> bool {
        let delta = match m.kind {
            crossterm::event::MouseEventKind::ScrollUp => -SCROLL_STEP_I32,
            crossterm::event::MouseEventKind::ScrollDown => SCROLL_STEP_I32,
            _ => return false,
        };
        let areas = self.overlay_areas();
        let counts = self.overlay_counts();
        let _ = with_active_table_handle(self.view, areas, counts, |mut handle| {
            handle.scroll_offset_by(delta);
            if let Some(row) = handle.row_at(m.column, m.row) {
                handle.select(row);
            }
        });
        true
    }

    fn overlay_row_at(&mut self, mouse_x: u16, mouse_y: u16) -> Option<usize> {
        let areas = self.overlay_areas();
        let counts = self.overlay_counts();
        with_active_table_handle(self.view, areas, counts, |handle| {
            handle.row_at(mouse_x, mouse_y)
        })
        .flatten()
    }

    fn overlay_areas(&self) -> OverlayAreas {
        OverlayAreas {
            full: self.layout.size,
            msg: self.layout.msg_area,
        }
    }

    fn overlay_counts(&self) -> OverlayRowCounts {
        let question_reviews = self
            .dispatch
            .tabs
            .get(*self.dispatch.active_tab)
            .and_then(|tab| tab.app.pending_question_review.as_ref())
            .map(|pending| pending.questions.len())
            .unwrap_or(0);
        OverlayRowCounts {
            tabs: self.dispatch.tabs.len(),
            jump: self.jump_rows.len(),
            models: self.dispatch.registry.models.len(),
            prompts: self.dispatch.prompt_registry.prompts.len(),
            question_reviews,
            help: help_rows_len(),
        }
    }
}

pub(crate) fn clamp_overlay_tables(view: &mut ViewState, state: &RenderState<'_>, jump_len: usize) {
    let areas = OverlayAreas {
        full: state.full_area,
        msg: state.msg_area,
    };
    let counts = OverlayRowCounts {
        tabs: state.tabs.len(),
        jump: jump_len,
        models: state.models.len(),
        prompts: state.prompts.len(),
        question_reviews: state
            .tabs
            .get(state.active_tab)
            .and_then(|tab| tab.app.pending_question_review.as_ref())
            .map(|pending| pending.questions.len())
            .unwrap_or(0),
        help: help_rows_len(),
    };
    let _ = with_active_table_handle(view, areas, counts, |mut handle| handle.clamp());
}
