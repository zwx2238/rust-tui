use crate::render::{RenderTheme, render_markdown_lines};
use crate::ui::draw::style::{base_fg, base_style, selection_bg};
use crate::ui::state::{CodeExecHover, PendingCodeExec};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

const MIN_POPUP_WIDTH: u16 = 40;
const MIN_POPUP_HEIGHT: u16 = 8;
const OUTER_MARGIN: u16 = 2;
#[derive(Copy, Clone)]
pub(crate) struct CodeExecPopupLayout {
    pub(crate) popup: Rect,
    pub(crate) code_text_area: Rect,
    pub(crate) code_scrollbar_area: Rect,
    pub(crate) stdout_text_area: Rect,
    pub(crate) stdout_scrollbar_area: Rect,
    pub(crate) stderr_text_area: Rect,
    pub(crate) stderr_scrollbar_area: Rect,
    pub(crate) approve_btn: Rect,
    pub(crate) deny_btn: Rect,
    pub(crate) stop_btn: Rect,
    pub(crate) exit_btn: Rect,
}

pub(crate) fn code_exec_popup_layout(area: Rect) -> CodeExecPopupLayout {
    let safe = Rect {
        x: area.x.saturating_add(OUTER_MARGIN),
        y: area.y.saturating_add(OUTER_MARGIN),
        width: area.width.saturating_sub(OUTER_MARGIN.saturating_mul(2)),
        height: area.height.saturating_sub(OUTER_MARGIN.saturating_mul(2)),
    };
    let width = (safe.width * 75 / 100)
        .max(MIN_POPUP_WIDTH)
        .min(safe.width.saturating_sub(2).max(MIN_POPUP_WIDTH));
    let height = (safe.height * 65 / 100)
        .max(MIN_POPUP_HEIGHT)
        .min(safe.height.saturating_sub(2).max(MIN_POPUP_HEIGHT));
    let x = safe.x + (safe.width.saturating_sub(width)) / 2;
    let y = safe.y + (safe.height.saturating_sub(height)) / 2;
    let popup = Rect {
        x,
        y,
        width,
        height,
    };
    let inner = Rect {
        x: popup.x.saturating_add(1),
        y: popup.y.saturating_add(1),
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(2),
    };
    let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(3)]).split(inner);
    let body = chunks[0];
    let actions_area = chunks[1];
    let body_cols = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(body);
    let code_text_area = Rect {
        x: body_cols[0].x,
        y: body_cols[0].y,
        width: body_cols[0].width.saturating_sub(1),
        height: body_cols[0].height,
    };
    let code_scrollbar_area = Rect {
        x: body_cols[0].x.saturating_add(body_cols[0].width.saturating_sub(1)),
        y: body_cols[0].y,
        width: 1,
        height: body_cols[0].height,
    };
    let out_chunks = Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(body_cols[1]);
    let stdout_text_area = Rect {
        x: out_chunks[0].x,
        y: out_chunks[0].y,
        width: out_chunks[0].width.saturating_sub(1),
        height: out_chunks[0].height,
    };
    let stdout_scrollbar_area = Rect {
        x: out_chunks[0].x.saturating_add(out_chunks[0].width.saturating_sub(1)),
        y: out_chunks[0].y,
        width: 1,
        height: out_chunks[0].height,
    };
    let stderr_text_area = Rect {
        x: out_chunks[1].x,
        y: out_chunks[1].y,
        width: out_chunks[1].width.saturating_sub(1),
        height: out_chunks[1].height,
    };
    let stderr_scrollbar_area = Rect {
        x: out_chunks[1].x.saturating_add(out_chunks[1].width.saturating_sub(1)),
        y: out_chunks[1].y,
        width: 1,
        height: out_chunks[1].height,
    };
    let gap = 2u16;
    let btn_width = actions_area
        .width
        .saturating_sub(gap)
        .saturating_div(2)
        .max(6);
    let approve_btn = Rect {
        x: actions_area.x,
        y: actions_area.y,
        width: btn_width,
        height: actions_area.height,
    };
    let deny_btn = Rect {
        x: actions_area.x.saturating_add(btn_width + gap),
        y: actions_area.y,
        width: actions_area
            .width
            .saturating_sub(btn_width + gap)
            .max(btn_width),
        height: actions_area.height,
    };
    let stop_btn = Rect {
        x: actions_area.x,
        y: actions_area.y,
        width: actions_area.width,
        height: actions_area.height,
    };
    let exit_btn = Rect {
        x: actions_area.x,
        y: actions_area.y,
        width: actions_area.width,
        height: actions_area.height,
    };
    CodeExecPopupLayout {
        popup,
        code_text_area,
        code_scrollbar_area,
        stdout_text_area,
        stdout_scrollbar_area,
        stderr_text_area,
        stderr_scrollbar_area,
        approve_btn,
        deny_btn,
        stop_btn,
        exit_btn,
    }
}

pub(crate) fn code_exec_max_scroll(
    code: &str,
    width: u16,
    height: u16,
    theme: &crate::render::RenderTheme,
) -> usize {
    let md = code_to_markdown(code);
    let lines = render_markdown_lines(&md, width as usize, theme, false);
    let view_height = height.saturating_sub(1) as usize;
    lines.len().saturating_sub(view_height)
}

pub(crate) fn stdout_max_scroll(
    stdout: &str,
    width: u16,
    height: u16,
    theme: &crate::render::RenderTheme,
) -> usize {
    let md = stdout_to_markdown(stdout);
    let lines = render_markdown_lines(&md, width as usize, theme, false);
    let view_height = height.saturating_sub(1) as usize;
    lines.len().saturating_sub(view_height)
}

pub(crate) fn stderr_max_scroll(
    stderr: &str,
    width: u16,
    height: u16,
    theme: &crate::render::RenderTheme,
) -> usize {
    let md = stderr_to_markdown(stderr);
    let lines = render_markdown_lines(&md, width as usize, theme, false);
    let view_height = height.saturating_sub(1) as usize;
    lines.len().saturating_sub(view_height)
}

pub(crate) fn draw_code_exec_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingCodeExec,
    scroll: usize,
    stdout_scroll: usize,
    stderr_scroll: usize,
    hover: Option<CodeExecHover>,
    live: Option<&crate::ui::state::CodeExecLive>,
    theme: &RenderTheme,
) {
    let layout = code_exec_popup_layout(area);
    let max_x = area.x.saturating_add(area.width);
    let max_y = area.y.saturating_add(area.height);
    let mask_x = layout.popup.x.saturating_sub(OUTER_MARGIN).max(area.x);
    let mask_y = layout.popup.y.saturating_sub(OUTER_MARGIN).max(area.y);
    let mask_w = layout
        .popup
        .width
        .saturating_add(OUTER_MARGIN.saturating_mul(2))
        .min(max_x.saturating_sub(mask_x));
    let mask_h = layout
        .popup
        .height
        .saturating_add(OUTER_MARGIN.saturating_mul(2))
        .min(max_y.saturating_sub(mask_y));
    let mask = Rect {
        x: mask_x,
        y: mask_y,
        width: mask_w,
        height: mask_h,
    };
    f.render_widget(Clear, mask);
    let mask_block = Block::default().style(base_style(theme));
    f.render_widget(mask_block, mask);
    f.render_widget(Clear, layout.popup);
    let mask = Block::default().style(base_style(theme));
    f.render_widget(mask, layout.popup);
    let title = build_title(live);
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(vec![
            Span::styled(title, Style::default().fg(base_fg(theme)).add_modifier(Modifier::BOLD)),
        ]))
        .style(base_style(theme))
        .border_style(Style::default().fg(base_fg(theme)));
    f.render_widget(block, layout.popup);

    let (code_text, total_lines) = build_code_text(
        &pending.code,
        layout.code_text_area.width,
        layout.code_text_area.height,
        scroll,
        theme,
    );
    let code_para = Paragraph::new(code_text)
        .style(base_style(theme))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(code_para, layout.code_text_area);

    if total_lines > layout.code_text_area.height as usize {
        let viewport_len = layout.code_text_area.height as usize;
        let max_scroll = total_lines.saturating_sub(viewport_len);
        let mut state = ScrollbarState::new(max_scroll.saturating_add(1))
            .position(scroll.min(max_scroll))
            .viewport_content_length(viewport_len);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(base_fg(theme)))
            .track_style(Style::default().fg(base_fg(theme)));
        f.render_stateful_widget(scrollbar, layout.code_scrollbar_area, &mut state);
    }

    let (stdout_text, stdout_lines) = build_stdout_text(
        live.map(|l| l.stdout.as_str()),
        layout.stdout_text_area.width,
        layout.stdout_text_area.height,
        stdout_scroll,
        theme,
    );
    let stdout_para = Paragraph::new(stdout_text)
        .style(base_style(theme))
        .block(Block::default().borders(Borders::NONE).title_top("STDOUT"));
    f.render_widget(stdout_para, layout.stdout_text_area);

    if stdout_lines > layout.stdout_text_area.height as usize {
        let viewport_len = layout.stdout_text_area.height as usize;
        let max_scroll = stdout_lines.saturating_sub(viewport_len);
        let mut state = ScrollbarState::new(max_scroll.saturating_add(1))
            .position(stdout_scroll.min(max_scroll))
            .viewport_content_length(viewport_len);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(base_fg(theme)))
            .track_style(Style::default().fg(base_fg(theme)));
        f.render_stateful_widget(scrollbar, layout.stdout_scrollbar_area, &mut state);
    }

    let (stderr_text, stderr_lines) = build_stderr_text(
        live.map(|l| l.stderr.as_str()),
        layout.stderr_text_area.width,
        layout.stderr_text_area.height,
        stderr_scroll,
        theme,
    );
    let stderr_para = Paragraph::new(stderr_text)
        .style(base_style(theme))
        .block(Block::default().borders(Borders::NONE).title_top("STDERR"));
    f.render_widget(stderr_para, layout.stderr_text_area);

    if stderr_lines > layout.stderr_text_area.height as usize {
        let viewport_len = layout.stderr_text_area.height as usize;
        let max_scroll = stderr_lines.saturating_sub(viewport_len);
        let mut state = ScrollbarState::new(max_scroll.saturating_add(1))
            .position(stderr_scroll.min(max_scroll))
            .viewport_content_length(viewport_len);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(base_fg(theme)))
            .track_style(Style::default().fg(base_fg(theme)));
        f.render_stateful_widget(scrollbar, layout.stderr_scrollbar_area, &mut state);
    }

    let approve_style = match hover {
        Some(CodeExecHover::Approve) => {
            Style::default().bg(selection_bg(theme.bg)).fg(base_fg(theme)).add_modifier(Modifier::BOLD)
        }
        _ => base_style(theme),
    };
    let deny_style = match hover {
        Some(CodeExecHover::Deny) => {
            Style::default().bg(selection_bg(theme.bg)).fg(base_fg(theme)).add_modifier(Modifier::BOLD)
        }
        _ => base_style(theme),
    };
    let exit_style = match hover {
        Some(CodeExecHover::Exit) => {
            Style::default().bg(selection_bg(theme.bg)).fg(base_fg(theme)).add_modifier(Modifier::BOLD)
        }
        _ => base_style(theme),
    };
    let stop_style = match hover {
        Some(CodeExecHover::Stop) => {
            Style::default().bg(selection_bg(theme.bg)).fg(base_fg(theme)).add_modifier(Modifier::BOLD)
        }
        _ => base_style(theme),
    };

    let finished = live.map(|l| l.done).unwrap_or(false);
    let running = live.is_some() && !finished;
    if finished {
        let exit_block = Block::default().borders(Borders::ALL).style(exit_style);
        f.render_widget(exit_block, layout.exit_btn);
        f.render_widget(
            Paragraph::new(Line::from("退出"))
                .style(exit_style)
                .alignment(ratatui::layout::Alignment::Center),
            layout.exit_btn,
        );
        return;
    }
    if running {
        let stop_block = Block::default().borders(Borders::ALL).style(stop_style);
        f.render_widget(stop_block, layout.stop_btn);
        f.render_widget(
            Paragraph::new(Line::from("停止执行"))
                .style(stop_style)
                .alignment(ratatui::layout::Alignment::Center),
            layout.stop_btn,
        );
        return;
    }
    let approve_block = Block::default().borders(Borders::ALL).style(approve_style);
    let deny_block = Block::default().borders(Borders::ALL).style(deny_style);
    f.render_widget(approve_block, layout.approve_btn);
    f.render_widget(
        Paragraph::new(Line::from("确认执行"))
            .style(approve_style)
            .alignment(ratatui::layout::Alignment::Center),
        layout.approve_btn,
    );
    f.render_widget(deny_block, layout.deny_btn);
    f.render_widget(
        Paragraph::new(Line::from("取消拒绝"))
            .style(deny_style)
            .alignment(ratatui::layout::Alignment::Center),
        layout.deny_btn,
    );
}

fn build_code_text(
    code: &str,
    width: u16,
    height: u16,
    scroll: usize,
    theme: &RenderTheme,
) -> (Text<'static>, usize) {
    let md = code_to_markdown(code);
    let lines = render_markdown_lines(&md, width as usize, theme, false);
    let view_height = height.saturating_sub(1) as usize;
    let start = scroll.min(lines.len());
    let end = (start + view_height).min(lines.len());
    let slice = lines[start..end].to_vec();
    (Text::from(slice), lines.len())
}

fn build_stdout_text(
    output: Option<&str>,
    width: u16,
    height: u16,
    scroll: usize,
    theme: &RenderTheme,
) -> (Text<'static>, usize) {
    let stdout = output.unwrap_or("");
    let md = stdout_to_markdown(stdout);
    let lines = render_markdown_lines(&md, width as usize, theme, false);
    let view_height = height.saturating_sub(1) as usize;
    let start = scroll.min(lines.len());
    let end = (start + view_height).min(lines.len());
    let slice = lines[start..end].to_vec();
    (Text::from(slice), lines.len())
}

fn build_stderr_text(
    output: Option<&str>,
    width: u16,
    height: u16,
    scroll: usize,
    theme: &RenderTheme,
) -> (Text<'static>, usize) {
    let stderr = output.unwrap_or("");
    let md = stderr_to_markdown(stderr);
    let lines = render_markdown_lines(&md, width as usize, theme, false);
    let view_height = height.saturating_sub(1) as usize;
    let start = scroll.min(lines.len());
    let end = (start + view_height).min(lines.len());
    let slice = lines[start..end].to_vec();
    (Text::from(slice), lines.len())
}

fn code_to_markdown(code: &str) -> String {
    if code.trim().is_empty() {
        "```python\n(空)\n```".to_string()
    } else {
        let mut out = String::from("```python\n");
        out.push_str(code);
        if !code.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```");
        out
    }
}

fn stdout_to_markdown(stdout: &str) -> String {
    let mut out = String::new();
    out.push_str("```text\n");
    if stdout.trim().is_empty() {
        out.push_str("(空)\n");
    } else {
        out.push_str(stdout);
        if !stdout.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("```\n");
    out
}

fn stderr_to_markdown(stderr: &str) -> String {
    let mut out = String::new();
    out.push_str("```text\n");
    if stderr.trim().is_empty() {
        out.push_str("(空)\n");
    } else {
        out.push_str(stderr);
        if !stderr.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("```\n");
    out
}

fn build_title(live: Option<&crate::ui::state::CodeExecLive>) -> String {
    match live {
        Some(live) => {
            let elapsed = live.started_at.elapsed().as_secs_f32();
            let status = if live.done { "已完成" } else { "执行中" };
            format!("代码执行确认 · {} {:.1}s", status, elapsed)
        }
        None => "代码执行确认 · 等待确认".to_string(),
    }
}
