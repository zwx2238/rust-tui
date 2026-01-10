use crate::render::RenderTheme;
use crate::ui::draw::style::base_fg;
use crate::ui::tab_bar::build_tab_bar_view;
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use std::error::Error;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use super::helpers::{handle_tab_category_mouse_down, handle_tab_category_wheel};

pub(super) struct TabsWidget;

impl Widget for TabsWidget {
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
        _jump_rows: &[crate::ui::jump::JumpRow],
        rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        let result = handle_tab_category_mouse_down(ctx, layout, update, rect, *m)?;
        if result.handled {
            return Ok(result);
        }
        handle_tab_category_wheel(ctx, layout, update, rect, *m)
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        draw_tabs(
            frame.frame,
            rect,
            frame.state.tab_labels,
            frame.state.active_tab_pos,
            frame.state.theme,
            frame.state.startup_text,
        );
        Ok(())
    }
}

fn draw_tabs(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    labels: &[String],
    active_tab: usize,
    theme: &RenderTheme,
    startup_text: Option<&str>,
) {
    let view = build_tab_bar_view(labels, active_tab, area.width);
    let mut spans = build_tab_spans(&view, theme);
    append_startup_text(&mut spans, area.width as usize, startup_text, theme);
    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.bg));
    f.render_widget(paragraph, area);
}

fn build_tab_spans(
    view: &crate::ui::tab_bar::TabBarView,
    theme: &RenderTheme,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (i, item) in view.items.iter().enumerate() {
        let style = if item.active {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(base_fg(theme))
        };
        spans.push(Span::styled(item.label.clone(), style));
        if i + 1 < view.items.len() {
            spans.push(Span::styled("│", Style::default().fg(base_fg(theme))));
        }
    }
    spans
}

fn append_startup_text(
    spans: &mut Vec<Span<'static>>,
    width: usize,
    startup_text: Option<&str>,
    theme: &RenderTheme,
) {
    let Some(text) = startup_text else {
        return;
    };
    let cursor = spans.iter().map(|s| s.content.width()).sum::<usize>();
    let text_width = text.width();
    if width <= cursor + text_width {
        return;
    }
    let pad = width.saturating_sub(cursor + text_width);
    spans.push(Span::raw(" ".repeat(pad)));
    spans.push(Span::styled(
        text.to_string(),
        Style::default().fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White)),
    ));
}
