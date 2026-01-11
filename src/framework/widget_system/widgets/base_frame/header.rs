use crate::render::RenderTheme;
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use std::error::Error;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub(super) struct HeaderWidget;

impl Widget for HeaderWidget {
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
        _event: &crossterm::event::Event,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        Ok(EventResult::ignored())
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        draw_header(
            frame.frame,
            rect,
            frame.state.theme,
            frame.state.header_note,
        );
        Ok(())
    }
}

fn draw_header(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    theme: &RenderTheme,
    note: Option<&str>,
) {
    let style = Style::default()
        .bg(theme.bg)
        .fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White));
    let text = if let Some(note) = note {
        format!("deepchat  ·  {note}")
    } else {
        "deepchat".to_string()
    };
    let line = Line::from(Span::styled(text, style));
    let paragraph = Paragraph::new(line).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}
