use crate::render::RenderTheme;
use crate::ui::state::{App, Notice};
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
    let Some(notice) = app.notice.as_ref() else {
        return;
    };
    if Instant::now() >= notice.expires_at {
        app.notice = None;
        return;
    }
    if area.width < MIN_WIDTH as u16 + 2 || area.height < MIN_HEIGHT as u16 + 2 {
        return;
    }
    let max_width = area.width.saturating_sub(4).max(MIN_WIDTH as u16) as usize;
    let wrapped = wrap(&notice.text, max_width);
    let content_width = wrapped
        .iter()
        .map(|l| UnicodeWidthStr::width(l.as_ref()))
        .max()
        .unwrap_or(0);
    let popup_width = (content_width + 2).min(max_width).max(MIN_WIDTH) as u16;
    let popup_height = (wrapped.len() + 2)
        .min(area.height.saturating_sub(2) as usize)
        .max(MIN_HEIGHT) as u16;
    let x = area
        .x
        .saturating_add(area.width.saturating_sub(popup_width) / 2);
    let y = area.y.saturating_add(1);

    let mut config = PopupConfig::default();
    config.show_title = false;
    config.position = PopupPosition::Absolute {
        x,
        y,
        width: popup_width,
        height: popup_height,
    };
    config.border_style = Style::default().fg(Color::Yellow);
    config.background_style = Style::default()
        .bg(theme.bg)
        .fg(theme.fg.unwrap_or(Color::White));
    config.popup_background_style = Style::default().bg(theme.bg);

    let mut popup = PopupWidget::with_content(config, TextContent::new(notice.text.clone()));
    popup.show();
    popup.render(f, area);
}
