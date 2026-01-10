use crate::render::RenderTheme;
use crate::ui::overlay_table::{OverlayTable, centered_area, draw_overlay_table, header_style};
use crate::ui::state::{PendingQuestionReview, QuestionDecision};
use crate::ui::text_utils::{collapse_text, truncate_to_width};
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};
use unicode_width::UnicodeWidthStr;

const POPUP_MAX_HEIGHT: u16 = 20;

pub fn draw_question_review_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingQuestionReview,
    selected: usize,
    scroll: usize,
    theme: &RenderTheme,
) {
    let popup = question_review_popup_area(area, pending.questions.len());
    let table = build_question_review_table(pending, selected, scroll, theme, popup);
    draw_overlay_table(f, popup, table);
}

pub fn question_review_popup_area(area: Rect, rows: usize) -> Rect {
    centered_area(area, 90, rows, POPUP_MAX_HEIGHT)
}

fn build_question_review_table<'a>(
    pending: &PendingQuestionReview,
    selected: usize,
    scroll: usize,
    theme: &'a RenderTheme,
    popup: Rect,
) -> OverlayTable<'a> {
    let status_width = status_col_width();
    let header =
        Row::new(vec![Cell::from("状态"), Cell::from("问题")]).style(header_style(theme));
    let body = pending.questions.iter().map(|q| {
        let text = truncate_to_width(
            &collapse_text(&q.question),
            max_question_width(popup, status_width),
        );
        Row::new(vec![Cell::from(q.decision.label()), Cell::from(text)])
    });
    OverlayTable {
        title: Line::from(table_title(pending)),
        header,
        rows: body.collect(),
        widths: vec![Constraint::Length(status_width), Constraint::Min(10)],
        selected,
        scroll,
        theme,
    }
}

fn table_title(pending: &PendingQuestionReview) -> String {
    let (pending_count, approved, rejected) = decision_counts(pending);
    format!(
        "批量提问 · 未确认 {pending_count} · 通过 {approved} · 拒绝 {rejected} · 空格切换通过/拒绝 · Enter 提交 · Esc 取消"
    )
}

fn decision_counts(pending: &PendingQuestionReview) -> (usize, usize, usize) {
    let mut pending_count = 0usize;
    let mut approved = 0usize;
    let mut rejected = 0usize;
    for item in &pending.questions {
        match item.decision {
            QuestionDecision::Pending => pending_count += 1,
            QuestionDecision::Approved => approved += 1,
            QuestionDecision::Rejected => rejected += 1,
        }
    }
    (pending_count, approved, rejected)
}

fn status_col_width() -> u16 {
    let mut max = "状态".width();
    for label in [
        QuestionDecision::Pending,
        QuestionDecision::Approved,
        QuestionDecision::Rejected,
    ] {
        max = max.max(label.label().width());
    }
    (max + 2) as u16
}

fn max_question_width(area: Rect, status_width: u16) -> usize {
    area.width.saturating_sub(status_width).saturating_sub(4) as usize
}
