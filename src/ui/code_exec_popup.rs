use crate::render::RenderTheme;
use crate::ui::code_exec_popup_layout::{OUTER_MARGIN, code_exec_popup_layout};
use crate::ui::code_exec_popup_text::{build_code_text, build_stderr_text, build_stdout_text};
use crate::ui::draw::style::{base_fg, base_style, selection_bg};
use crate::ui::state::{CodeExecHover, CodeExecReasonTarget, PendingCodeExec};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use tui_textarea::TextArea;

pub(crate) fn draw_code_exec_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingCodeExec,
    scroll: usize,
    stdout_scroll: usize,
    stderr_scroll: usize,
    hover: Option<CodeExecHover>,
    reason_target: Option<CodeExecReasonTarget>,
    reason_input: &mut TextArea<'_>,
    live: Option<&crate::ui::state::CodeExecLive>,
    theme: &RenderTheme,
) {
    let layout = code_exec_popup_layout(area, reason_target.is_some());
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
        .title_top(Line::from(vec![Span::styled(
            title,
            Style::default()
                .fg(base_fg(theme))
                .add_modifier(Modifier::BOLD),
        )]))
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

    if let Some(target) = reason_target {
        draw_reason_input(f, layout.reason_input_area, reason_input, target, theme);
    }

    let button_style = |target: CodeExecHover| match hover {
        Some(h) if h == target => Style::default()
            .bg(selection_bg(theme.bg))
            .fg(base_fg(theme))
            .add_modifier(Modifier::BOLD),
        _ => base_style(theme),
    };
    let approve_style = button_style(CodeExecHover::Approve);
    let deny_style = button_style(CodeExecHover::Deny);
    let exit_style = button_style(CodeExecHover::Exit);
    let stop_style = button_style(CodeExecHover::Stop);

    let finished = live
        .map(|l| l.done || l.exit_code.is_some())
        .unwrap_or(false);
    let running = live.is_some() && !finished;
    if let Some(target) = reason_target {
        let confirm_style = button_style(CodeExecHover::ReasonConfirm);
        let back_style = button_style(CodeExecHover::ReasonBack);
        let confirm_label = match target {
            CodeExecReasonTarget::Deny => "确认取消",
            CodeExecReasonTarget::Stop => "确认中止",
        };
        let confirm_block = Block::default().borders(Borders::ALL).style(confirm_style);
        let back_block = Block::default().borders(Borders::ALL).style(back_style);
        f.render_widget(confirm_block, layout.approve_btn);
        f.render_widget(
            Paragraph::new(Line::from(confirm_label))
                .style(confirm_style)
                .alignment(ratatui::layout::Alignment::Center),
            layout.approve_btn,
        );
        f.render_widget(back_block, layout.deny_btn);
        f.render_widget(
            Paragraph::new(Line::from("返回"))
                .style(back_style)
                .alignment(ratatui::layout::Alignment::Center),
            layout.deny_btn,
        );
        return;
    }
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

fn build_title(live: Option<&crate::ui::state::CodeExecLive>) -> String {
    match live {
        Some(live) => {
            let finished = live.done || live.exit_code.is_some();
            if finished {
                let finished_at = live.finished_at.unwrap_or_else(std::time::Instant::now);
                let exec = finished_at.duration_since(live.started_at).as_secs_f32();
                let wait = finished_at.elapsed().as_secs_f32();
                format!("代码执行确认 · 已完成 {:.1}s | 等待 {:.1}s", exec, wait)
            } else {
                let elapsed = live.started_at.elapsed().as_secs_f32();
                format!("代码执行确认 · 执行中 {:.1}s", elapsed)
            }
        }
        None => "代码执行确认 · 等待确认".to_string(),
    }
}

fn draw_reason_input(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    input: &mut TextArea<'_>,
    target: CodeExecReasonTarget,
    theme: &RenderTheme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let title = match target {
        CodeExecReasonTarget::Deny => "取消原因(可选)",
        CodeExecReasonTarget::Stop => "中止原因(可选)",
    };
    let style = base_style(theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(title))
        .style(style);
    input.set_block(block);
    input.set_style(style);
    input.set_selection_style(Style::default().bg(selection_bg(theme.bg)));
    input.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    input.set_placeholder_text("可填写原因，留空使用默认提示");
    input.set_placeholder_style(Style::default().fg(base_fg(theme)));
    f.render_widget(&*input, area);
}
