use crate::render::RenderTheme;
use crate::framework::widget_system::runtime::state::{App, Notice};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use stakpak_popup_widget::{PopupConfig, PopupPosition, PopupWidget, TextContent};
use std::time::{Duration, Instant};
use textwrap::wrap;
use unicode_width::UnicodeWidthStr;

const NOTICE_TTL: Duration = Duration::from_secs(2);
const MIN_WIDTH: usize = 16;
const MIN_HEIGHT: usize = 3;

pub(crate) fn push_notice(app: &mut App, text: impl Into<String>) {
    app.notice = Some(Notice {
        text: text.into(),
        expires_at: Instant::now() + NOTICE_TTL,
    });
}

pub(crate) fn draw_notice(f: &mut Frame<'_>, area: Rect, app: &mut App, theme: &RenderTheme) {
    let Some(notice) = active_notice(app) else {
        return;
    };
    if !has_space(area) {
        return;
    }
    let popup = build_popup_config(area, &notice.text, theme);
    let mut widget = PopupWidget::with_content(popup, TextContent::new(notice.text.clone()));
    widget.show();
    widget.render(f, area);
}

fn active_notice(app: &mut App) -> Option<Notice> {
    let notice = app.notice.as_ref()?;
    if Instant::now() >= notice.expires_at {
        app.notice = None;
        return None;
    }
    Some(notice.clone())
}

fn has_space(area: Rect) -> bool {
    area.width >= MIN_WIDTH as u16 + 2 && area.height >= MIN_HEIGHT as u16 + 2
}

fn build_popup_config(area: Rect, text: &str, theme: &RenderTheme) -> PopupConfig {
    let (popup_width, popup_height) = measure_popup(area, text);
    let x = area
        .x
        .saturating_add(area.width.saturating_sub(popup_width) / 2);
    let y = area.y.saturating_add(1);
    PopupConfig {
        show_title: false,
        position: PopupPosition::Absolute {
            x,
            y,
            width: popup_width,
            height: popup_height,
        },
        border_style: Style::default().fg(Color::Yellow),
        background_style: Style::default()
            .bg(theme.bg)
            .fg(theme.fg.unwrap_or(Color::White)),
        popup_background_style: Style::default().bg(theme.bg),
        ..Default::default()
    }
}

fn measure_popup(area: Rect, text: &str) -> (u16, u16) {
    let max_width = area.width.saturating_sub(4).max(MIN_WIDTH as u16) as usize;
    let wrapped = wrap(text, max_width);
    let content_width = wrapped
        .iter()
        .map(|l| UnicodeWidthStr::width(l.as_ref()))
        .max()
        .unwrap_or(0);
    let popup_width = (content_width + 2).min(max_width).max(MIN_WIDTH) as u16;
    let popup_height = (wrapped.len() + 2)
        .min(area.height.saturating_sub(2) as usize)
        .max(MIN_HEIGHT) as u16;
    (popup_width, popup_height)
}
