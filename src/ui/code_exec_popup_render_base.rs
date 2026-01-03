use crate::render::RenderTheme;
use crate::ui::code_exec_popup_layout::{CodeExecPopupLayout, OUTER_MARGIN};
use crate::ui::code_exec_popup_text::{build_code_text, build_stderr_text, build_stdout_text};
use crate::ui::draw::style::{base_fg, base_style};
use crate::ui::state::PendingCodeExec;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

pub(crate) fn popup_mask(area: Rect, popup: Rect) -> Rect {
    let max_x = area.x.saturating_add(area.width);
    let max_y = area.y.saturating_add(area.height);
    let mask_x = popup.x.saturating_sub(OUTER_MARGIN).max(area.x);
    let mask_y = popup.y.saturating_sub(OUTER_MARGIN).max(area.y);
    let mask_w = popup
        .width
        .saturating_add(OUTER_MARGIN.saturating_mul(2))
        .min(max_x.saturating_sub(mask_x));
    let mask_h = popup
        .height
        .saturating_add(OUTER_MARGIN.saturating_mul(2))
        .min(max_y.saturating_sub(mask_y));
    Rect {
        x: mask_x,
        y: mask_y,
        width: mask_w,
        height: mask_h,
    }
}

pub(crate) fn render_mask(f: &mut ratatui::Frame<'_>, theme: &RenderTheme, mask: Rect) {
    f.render_widget(Clear, mask);
    let mask_block = Block::default().style(base_style(theme));
    f.render_widget(mask_block, mask);
}

pub(crate) fn render_popup_base(
    f: &mut ratatui::Frame<'_>,
    theme: &RenderTheme,
    popup: Rect,
    title: &str,
) {
    f.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(vec![Span::styled(
            title,
            Style::default()
                .fg(base_fg(theme))
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]))
        .style(base_style(theme))
        .border_style(Style::default().fg(base_fg(theme)));
    f.render_widget(block, popup);
}

pub(crate) fn render_code_panel(
    f: &mut ratatui::Frame<'_>,
    theme: &RenderTheme,
    pending: &PendingCodeExec,
    layout: CodeExecPopupLayout,
    scroll: usize,
) {
    let (text, total_lines) = build_code_text(
        &pending.code,
        layout.code_text_area.width,
        layout.code_text_area.height,
        scroll,
        theme,
    );
    render_text_panel(
        f,
        TextPanelParams {
            theme,
            text,
            area: layout.code_text_area,
            scrollbar_area: layout.code_scrollbar_area,
            total_lines,
            scroll,
            title: None,
        },
    );
}

pub(crate) fn render_stdout_panel(
    f: &mut ratatui::Frame<'_>,
    theme: &RenderTheme,
    layout: CodeExecPopupLayout,
    live: Option<&crate::ui::state::CodeExecLive>,
    scroll: usize,
) {
    let (text, total_lines) = build_stdout_text(
        live.map(|l| l.stdout.as_str()),
        layout.stdout_text_area.width,
        layout.stdout_text_area.height,
        scroll,
        theme,
    );
    render_text_panel(
        f,
        TextPanelParams {
            theme,
            text,
            area: layout.stdout_text_area,
            scrollbar_area: layout.stdout_scrollbar_area,
            total_lines,
            scroll,
            title: Some("STDOUT"),
        },
    );
}

pub(crate) fn render_stderr_panel(
    f: &mut ratatui::Frame<'_>,
    theme: &RenderTheme,
    layout: CodeExecPopupLayout,
    live: Option<&crate::ui::state::CodeExecLive>,
    scroll: usize,
) {
    let (text, total_lines) = build_stderr_text(
        live.map(|l| l.stderr.as_str()),
        layout.stderr_text_area.width,
        layout.stderr_text_area.height,
        scroll,
        theme,
    );
    render_text_panel(
        f,
        TextPanelParams {
            theme,
            text,
            area: layout.stderr_text_area,
            scrollbar_area: layout.stderr_scrollbar_area,
            total_lines,
            scroll,
            title: Some("STDERR"),
        },
    );
}

struct TextPanelParams<'a> {
    theme: &'a RenderTheme,
    text: Text<'a>,
    area: Rect,
    scrollbar_area: Rect,
    total_lines: usize,
    scroll: usize,
    title: Option<&'a str>,
}

fn render_text_panel(f: &mut ratatui::Frame<'_>, params: TextPanelParams<'_>) {
    let block = match params.title {
        Some(title) => Block::default().borders(Borders::NONE).title_top(title),
        None => Block::default().borders(Borders::NONE),
    };
    let para = Paragraph::new(params.text)
        .style(base_style(params.theme))
        .block(block);
    f.render_widget(para, params.area);
    render_scrollbar_if_needed(
        f,
        params.theme,
        params.area,
        params.scrollbar_area,
        params.total_lines,
        params.scroll,
    );
}

fn render_scrollbar_if_needed(
    f: &mut ratatui::Frame<'_>,
    theme: &RenderTheme,
    area: Rect,
    scrollbar_area: Rect,
    total_lines: usize,
    scroll: usize,
) {
    if total_lines <= area.height as usize {
        return;
    }
    let viewport_len = area.height as usize;
    let max_scroll = total_lines.saturating_sub(viewport_len);
    let mut state = ScrollbarState::new(max_scroll.saturating_add(1))
        .position(scroll.min(max_scroll))
        .viewport_content_length(viewport_len);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(Style::default().fg(base_fg(theme)))
        .track_style(Style::default().fg(base_fg(theme)));
    f.render_stateful_widget(scrollbar, scrollbar_area, &mut state);
}
