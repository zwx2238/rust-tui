mod data;
mod rows;

pub(crate) use data::{SummaryRow, SummarySort, build_summary_rows, sort_summary_rows};

use crate::render::RenderTheme;
use crate::framework::widget_system::draw::layout::{inner_area, layout_chunks};
use crate::framework::widget_system::draw::style::base_fg;
use crate::framework::widget_system::widgets::jump::JumpRow;
use crate::framework::widget_system::widgets::overlay_table::{OverlayTable, draw_overlay_table, header_style};
use crate::framework::widget_system::layout::compute_sidebar_width;
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::widgets::tab_bar::build_tab_bar_view;
use crate::framework::widget_system::interaction::text_utils::truncate_to_width;
use std::error::Error;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row};
use unicode_width::UnicodeWidthStr;

use super::super::bindings::bind_event;
use super::super::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::lifecycle::{EventResult, Widget};
use super::overlay_table::{OverlayTableController, clamp_overlay_tables};

pub(crate) struct SummaryWidget {
    _private: (),
}

impl SummaryWidget {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl Widget for SummaryWidget {
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
        jump_rows: &[JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let binding = bind_event(ctx, layout, update);
        let mut controller = OverlayTableController {
            dispatch: binding.dispatch,
            layout: binding.layout,
            view: binding.view,
            jump_rows,
        };
        controller.handle_event(event)
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        clamp_overlay_tables(frame.view, frame.state, frame.jump_rows.len());
        let layout = summary_layout(rect, frame.state.categories);
        let rows = summary_rows(frame, &layout);
        draw_summary_layout(frame, &layout);
        draw_summary_table(frame, &layout, &rows);
        update_summary_order(frame, &rows);
        Ok(())
    }
}

fn summary_layout(
    rect: ratatui::layout::Rect,
    categories: &[String],
) -> SummaryLayout {
    let size = ratatui::layout::Size {
        width: rect.width,
        height: rect.height,
    };
    build_summary_layout(size, categories)
}

fn summary_rows(
    frame: &WidgetFrame<'_, '_, '_, '_>,
    layout: &SummaryLayout,
) -> Vec<SummaryRow> {
    let mut rows = build_summary_rows(frame.state.tabs(), layout.max_latest_width.max(10));
    sort_summary_rows(&mut rows, frame.view.summary_sort);
    rows
}

fn draw_summary_layout(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    layout: &SummaryLayout,
) {
    let params = DrawSummaryLayoutParams {
        f: frame.frame,
        theme: frame.state.theme,
        tab_labels: frame.state.tab_labels,
        active_tab_pos: frame.state.active_tab_pos,
        categories: frame.state.categories,
        active_category: frame.state.active_category,
        header_note: frame.state.header_note,
        startup_text: frame.state.startup_text,
        header_area: layout.header_area,
        category_area: layout.category_area,
        tabs_area: layout.tabs_area,
        footer_area: layout.footer_area,
    };
    draw_summary_layout_base(params);
}

fn draw_summary_table(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    layout: &SummaryLayout,
    rows: &[SummaryRow],
) {
    draw_summary_table_base(
        frame.frame,
        layout.body_area,
        rows,
        frame.view.summary.selected,
        frame.view.summary.scroll,
        frame.state.theme,
        frame.view.summary_sort,
    );
}

fn update_summary_order(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    rows: &[SummaryRow],
) {
    frame.view.summary_order = rows.iter().map(|r| r.tab_index).collect();
}

struct SummaryLayout {
    header_area: Rect,
    category_area: Rect,
    tabs_area: Rect,
    body_area: Rect,
    footer_area: Rect,
    max_latest_width: usize,
}

struct DrawSummaryLayoutParams<'a, 'b> {
    f: &'a mut ratatui::Frame<'b>,
    theme: &'a RenderTheme,
    tab_labels: &'a [String],
    active_tab_pos: usize,
    categories: &'a [String],
    active_category: usize,
    header_note: Option<&'a str>,
    startup_text: Option<&'a str>,
    header_area: Rect,
    category_area: Rect,
    tabs_area: Rect,
    footer_area: Rect,
}

fn build_summary_layout(size: ratatui::layout::Size, categories: &[String]) -> SummaryLayout {
    let size = Rect::new(0, 0, size.width, size.height);
    let sidebar_width = compute_sidebar_width(categories, size.width);
    let (header_area, category_area, tabs_area, body_area, _input_area, footer_area) =
        layout_chunks(size, 0, sidebar_width);
    SummaryLayout {
        header_area,
        category_area,
        tabs_area,
        body_area,
        footer_area,
        max_latest_width: max_latest_question_width(body_area),
    }
}

fn draw_summary_layout_base<'a, 'b>(params: DrawSummaryLayoutParams<'a, 'b>) {
    draw_header(
        params.f,
        params.header_area,
        params.theme,
        params.header_note,
    );
    draw_categories(
        params.f,
        params.category_area,
        params.categories,
        params.active_category,
        params.theme,
    );
    draw_tabs(
        params.f,
        params.tabs_area,
        params.tab_labels,
        params.active_tab_pos,
        params.theme,
        params.startup_text,
    );
    draw_footer(params.f, params.footer_area, params.theme, false, false);
}

fn max_latest_question_width(body_area: Rect) -> usize {
    inner_area(body_area, 1, 1).width.saturating_sub(40) as usize
}

fn draw_summary_table_base(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    rows: &[SummaryRow],
    selected_row: usize,
    scroll: usize,
    theme: &RenderTheme,
    sort: SummarySort,
) {
    let popup = build_summary_table(rows, selected_row, scroll, theme, sort);
    draw_overlay_table(f, area, popup);
}

fn build_summary_table<'a>(
    rows: &'a [SummaryRow],
    selected_row: usize,
    scroll: usize,
    theme: &'a RenderTheme,
    sort: SummarySort,
) -> OverlayTable<'a> {
    let header = summary_header(theme);
    let body = summary_body(rows);
    OverlayTable {
        title: Line::from(summary_title(sort)),
        header,
        rows: body.collect(),
        widths: summary_widths(),
        selected: selected_row,
        scroll,
        theme,
    }
}

fn summary_header(theme: &RenderTheme) -> Row<'static> {
    Row::new(vec![
        Cell::from("对话"),
        Cell::from("分类"),
        Cell::from("消息数"),
        Cell::from("状态"),
        Cell::from("执行中"),
        Cell::from("最新提问"),
    ])
    .style(header_style(theme))
}

fn summary_body<'a>(
    rows: &'a [SummaryRow],
) -> impl Iterator<Item = Row<'a>> + 'a {
    rows.iter().map(|row| {
        Row::new(vec![
            Cell::from(row.tab_id.to_string()),
            Cell::from(row.category.clone()),
            Cell::from(row.message_count.to_string()),
            Cell::from(row.status),
            Cell::from(if row.exec_pending { "是" } else { "否" }),
            Cell::from(row.latest_user.clone()),
        ])
    })
}

fn summary_widths() -> Vec<Constraint> {
    vec![
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Min(10),
    ]
}

fn summary_title(sort: SummarySort) -> &'static str {
    match sort {
        SummarySort::TabOrder => "汇总页 · F1 退出 · Enter 进入 · S 排序(默认)",
        SummarySort::ExecTime => {
            "汇总页 · F1 退出 · Enter 进入 · S 排序(执行中)"
        }
    }
}

fn draw_header(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    theme: &RenderTheme,
    note: Option<&str>,
) {
    let style = Style::default()
        .bg(theme.bg)
        .fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White));
    let text = if let Some(note) = note {
        format!("deepchat  ·  {note}")
    } else {
        "deepchat".to_string()
    };
    let line = Line::from(Span::styled(text, style));
    let paragraph = Paragraph::new(line).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
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
    view: &crate::framework::widget_system::widgets::tab_bar::TabBarView,
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

fn draw_categories(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    categories: &[String],
    active: usize,
    theme: &RenderTheme,
) {
    let mut lines = Vec::new();
    let width = area.width.saturating_sub(2).max(1) as usize;
    for (idx, name) in categories.iter().enumerate() {
        let prefix = if idx == active { "● " } else { "  " };
        let label = truncate_to_width(name, width.saturating_sub(2));
        let text = format!("{prefix}{label}");
        let style = if idx == active {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(base_fg(theme))
        };
        lines.push(Line::from(Span::styled(text, style)));
    }
    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(theme.bg));
    f.render_widget(paragraph, area);
}
