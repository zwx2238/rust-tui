use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::error::Error;

use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};

pub(crate) struct ButtonWidget {
    label: String,
    style: Style,
    rect: Rect,
    visible: bool,
    bordered: bool,
}

impl ButtonWidget {
    pub(crate) fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            style: Style::default(),
            rect: Rect::new(0, 0, 0, 0),
            visible: true,
            bordered: true,
        }
    }

    pub(crate) fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    pub(crate) fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    pub(crate) fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub(crate) fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub(crate) fn set_bordered(&mut self, bordered: bool) {
        self.bordered = bordered;
    }

    fn contains(&self, column: u16, row: u16) -> bool {
        column >= self.rect.x
            && column < self.rect.x.saturating_add(self.rect.width)
            && row >= self.rect.y
            && row < self.rect.y.saturating_add(self.rect.height)
    }
}

impl Widget for ButtonWidget {
    fn place(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &mut FrameLayout,
        rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.rect = rect;
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
        _ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _rect: Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        if !self.visible {
            return Ok(EventResult::ignored());
        }
        let crossterm::event::Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        if !matches!(m.kind, crossterm::event::MouseEventKind::Down(_)) {
            return Ok(EventResult::ignored());
        }
        if !self.contains(m.column, m.row) {
            return Ok(EventResult::ignored());
        }
        Ok(EventResult::handled())
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        if !self.visible || self.rect.width == 0 || self.rect.height == 0 {
            return Ok(());
        }
        if self.bordered {
            let block = Block::default().borders(Borders::ALL).style(self.style);
            frame.frame.render_widget(block, self.rect);
        }
        frame.frame.render_widget(
            Paragraph::new(Line::from(self.label.clone()))
                .style(self.style)
                .alignment(ratatui::layout::Alignment::Center),
            self.rect,
        );
        Ok(())
    }
}
