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
    pub(crate) output_text_area: Rect,
    pub(crate) output_scrollbar_area: Rect,
    pub(crate) approve_btn: Rect,
    pub(crate) deny_btn: Rect,
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
    let chunks = Layout::vertical([
        Constraint::Percentage(45),
        Constraint::Percentage(35),
        Constraint::Length(3),
    ])
    .split(inner);
    let code_text_area = Rect {
        x: chunks[0].x,
        y: chunks[0].y,
        width: chunks[0].width.saturating_sub(1),
        height: chunks[0].height,
    };
    let code_scrollbar_area = Rect {
        x: chunks[0].x.saturating_add(chunks[0].width.saturating_sub(1)),
        y: chunks[0].y,
        width: 1,
        height: chunks[0].height,
    };
    let output_text_area = Rect {
        x: chunks[1].x,
        y: chunks[1].y,
        width: chunks[1].width.saturating_sub(1),
        height: chunks[1].height,
    };
    let output_scrollbar_area = Rect {
        x: chunks[1].x.saturating_add(chunks[1].width.saturating_sub(1)),
        y: chunks[1].y,
        width: 1,
        height: chunks[1].height,
    };
    let actions_area = chunks[2];
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
        output_text_area,
        output_scrollbar_area,
        approve_btn,
        deny_btn,
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

pub(crate) fn output_max_scroll(
    stdout: &str,
    stderr: &str,
    width: u16,
    height: u16,
    theme: &crate::render::RenderTheme,
) -> usize {
    let md = output_to_markdown(stdout, stderr);
    let lines = render_markdown_lines(&md, width as usize, theme, false);
    let view_height = height.saturating_sub(1) as usize;
    lines.len().saturating_sub(view_height)
}

pub(crate) fn draw_code_exec_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingCodeExec,
    scroll: usize,
    output_scroll: usize,
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

    let (output_text, output_lines) = build_output_text(
        live.map(|l| (l.stdout.as_str(), l.stderr.as_str())),
        layout.output_text_area.width,
        layout.output_text_area.height,
        output_scroll,
        theme,
    );
    let output_para = Paragraph::new(output_text)
        .style(base_style(theme))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(output_para, layout.output_text_area);

    if output_lines > layout.output_text_area.height as usize {
        let viewport_len = layout.output_text_area.height as usize;
        let max_scroll = output_lines.saturating_sub(viewport_len);
        let mut state = ScrollbarState::new(max_scroll.saturating_add(1))
            .position(output_scroll.min(max_scroll))
            .viewport_content_length(viewport_len);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(base_fg(theme)))
            .track_style(Style::default().fg(base_fg(theme)));
        f.render_stateful_widget(scrollbar, layout.output_scrollbar_area, &mut state);
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
        let wait_block = Block::default().borders(Borders::ALL).style(base_style(theme));
        f.render_widget(wait_block, layout.exit_btn);
        f.render_widget(
            Paragraph::new(Line::from("执行中…"))
                .style(base_style(theme))
                .alignment(ratatui::layout::Alignment::Center),
            layout.exit_btn,
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

fn build_output_text(
    output: Option<(&str, &str)>,
    width: u16,
    height: u16,
    scroll: usize,
    theme: &RenderTheme,
) -> (Text<'static>, usize) {
    let (stdout, stderr) = output.unwrap_or(("", ""));
    let md = output_to_markdown(stdout, stderr);
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

fn output_to_markdown(stdout: &str, stderr: &str) -> String {
    let mut out = String::new();
    out.push_str("stdout:\n```text\n");
    if stdout.trim().is_empty() {
        out.push_str("(空)\n");
    } else {
        out.push_str(stdout);
        if !stdout.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("```\nstderr:\n```text\n");
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
