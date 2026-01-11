use crate::render::RenderTheme;
use crate::framework::widget_system::draw::style::base_fg;
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use std::error::Error;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub(super) struct FooterWidget;

impl Widget for FooterWidget {
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
        _jump_rows: &[crate::framework::widget_system::widgets::jump::JumpRow],
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
        if let Some(app) = frame.state.active_app() {
            draw_footer(
                frame.frame,
                rect,
                frame.state.theme,
                app.nav_mode,
                app.follow,
            );
        }
        Ok(())
    }
}

fn draw_footer(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    theme: &RenderTheme,
    nav_mode: bool,
    follow: bool,
) {
    let time = chrono::Local::now().format("%H:%M:%S").to_string();
    let mut parts = vec![time];
    if nav_mode {
        parts.push("NAV".to_string());
    }
    let follow_text = if follow { "追底:开" } else { "追底:关" };
    parts.push(follow_text.to_string());
    let text = parts.join("  ");
    let style = Style::default().bg(theme.bg).fg(base_fg(theme));
    let line = Line::from(Span::styled(text, style));
    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}
