use crate::render::RenderTheme;
use crate::ui::draw::style::base_style;
use crate::ui::overlay_table::{OverlayTable, centered_area, draw_overlay_table, header_style};
use crate::ui::state::{PendingQuestionItem, PendingQuestionReview, QuestionDecision};
use crate::ui::text_utils::{collapse_text, truncate_to_width};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row};
use textwrap::Options;
use unicode_width::UnicodeWidthStr;

const POPUP_MAX_HEIGHT: u16 = 24;
const LIST_PERCENT: u16 = 38;
const MIN_LIST_WIDTH: u16 = 26;
const MIN_DETAIL_WIDTH: u16 = 32;

pub fn draw_question_review_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingQuestionReview,
    selected: usize,
    list_scroll: usize,
    detail_scroll: &mut usize,
    theme: &RenderTheme,
) {
    let popup = question_review_popup_area(area, pending.questions.len());
    let (list_area, detail_area) = question_review_layout(popup);
    f.render_widget(Clear, popup);
    f.render_widget(Block::default().style(base_style(theme)), popup);
    draw_question_review_list(f, list_area, pending, selected, list_scroll, theme);
    draw_question_detail(f, detail_area, pending, selected, detail_scroll, theme);
}

pub fn question_review_popup_area(area: Rect, rows: usize) -> Rect {
    centered_area(area, 96, rows, POPUP_MAX_HEIGHT)
}

pub fn question_review_list_area(area: Rect, rows: usize) -> Rect {
    let popup = question_review_popup_area(area, rows);
    question_review_layout(popup).0
}

fn question_review_layout(area: Rect) -> (Rect, Rect) {
    let list_width = list_width(area.width);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(list_width), Constraint::Min(10)])
        .split(area);
    (chunks[0], chunks[1])
}

fn list_width(total: u16) -> u16 {
    let desired = total.saturating_mul(LIST_PERCENT) / 100;
    let max_list = total.saturating_sub(MIN_DETAIL_WIDTH).max(MIN_LIST_WIDTH);
    desired.clamp(MIN_LIST_WIDTH, max_list)
}

fn draw_question_review_list(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingQuestionReview,
    selected: usize,
    list_scroll: usize,
    theme: &RenderTheme,
) {
    let table = build_question_review_table(pending, selected, list_scroll, theme, area);
    draw_overlay_table(f, area, table);
}

fn build_question_review_table<'a>(
    pending: &PendingQuestionReview,
    selected: usize,
    list_scroll: usize,
    theme: &'a RenderTheme,
    area: Rect,
) -> OverlayTable<'a> {
    let status_width = status_col_width();
    let model_width = model_col_width(pending);
    let question_width = max_question_width(area, status_width, model_width);
    let header = table_header(theme);
    let body = table_rows(pending, question_width);
    OverlayTable {
        title: Line::from(table_title(pending)),
        header,
        rows: body,
        widths: vec![
            Constraint::Length(status_width),
            Constraint::Length(model_width),
            Constraint::Min(10),
        ],
        selected,
        scroll: list_scroll,
        theme,
    }
}

fn table_header(theme: &RenderTheme) -> Row<'static> {
    Row::new(vec![
        Cell::from("状态"),
        Cell::from("模型"),
        Cell::from("问题"),
    ])
    .style(header_style(theme))
}

fn table_rows(pending: &PendingQuestionReview, question_width: usize) -> Vec<Row<'static>> {
    pending
        .questions
        .iter()
        .map(|q| {
            let text = truncate_to_width(&collapse_text(&q.question), question_width);
            Row::new(vec![
                Cell::from(q.decision.label()),
                Cell::from(q.model_key.clone()),
                Cell::from(text),
            ])
        })
        .collect()
}

fn draw_question_detail(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    pending: &PendingQuestionReview,
    selected: usize,
    detail_scroll: &mut usize,
    theme: &RenderTheme,
) {
    let Some(item) = pending.questions.get(selected) else {
        return;
    };
    let width = area.width.saturating_sub(2);
    let lines = detail_lines(item, width);
    let max_scroll = detail_max_scroll(lines.len(), area);
    if *detail_scroll > max_scroll {
        *detail_scroll = max_scroll;
    }
    let text = Text::from(lines.into_iter().map(Line::from).collect::<Vec<_>>());
    let block = Block::default()
        .borders(Borders::ALL)
        .title("问题详情");
    let paragraph = Paragraph::new(text)
        .block(block)
        .style(base_style(theme))
        .scroll((*detail_scroll as u16, 0));
    f.render_widget(paragraph, area);
}

fn detail_lines(item: &PendingQuestionItem, width: u16) -> Vec<String> {
    let mut out = Vec::new();
    let header = format!("状态: {}  模型: {}", item.decision.label(), item.model_key);
    out.extend(wrap_text_lines(&header, width));
    out.push(String::new());
    out.extend(wrap_text_lines(&item.question, width));
    out
}

fn wrap_text_lines(text: &str, width: u16) -> Vec<String> {
    let width = width.max(1) as usize;
    let options = Options::new(width).break_words(true);
    let mut out = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            out.push(String::new());
            continue;
        }
        for wrapped in textwrap::wrap(line, &options) {
            out.push(wrapped.into_owned());
        }
    }
    if text.ends_with('\n') {
        out.push(String::new());
    }
    out
}

fn detail_max_scroll(lines: usize, area: Rect) -> usize {
    let view_height = area.height.saturating_sub(2) as usize;
    lines.saturating_sub(view_height)
}

fn table_title(pending: &PendingQuestionReview) -> String {
    let (pending_count, approved, rejected) = decision_counts(pending);
    format!(
        "批量提问 · 未确认 {pending_count} · 通过 {approved} · 拒绝 {rejected} · A 全通过 · m/Shift+M 切模型 · Alt+M 全同模型 · PgUp/PgDn 滚动详情 · Enter 提交 · Esc 取消"
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

fn model_col_width(pending: &PendingQuestionReview) -> u16 {
    let mut max = "模型".width();
    for item in &pending.questions {
        max = max.max(item.model_key.width());
    }
    (max + 2).min(20) as u16
}

fn max_question_width(area: Rect, status_width: u16, model_width: u16) -> usize {
    area.width
        .saturating_sub(status_width)
        .saturating_sub(model_width)
        .saturating_sub(6) as usize
}
