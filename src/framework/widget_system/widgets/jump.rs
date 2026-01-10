use crate::render::RenderTheme;
use crate::ui::draw::style::base_fg;
use crate::ui::jump::{JumpRow, build_jump_rows, max_preview_width};
use crate::ui::overlay_table::{OverlayTable, draw_overlay_table, header_style};
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::tab_bar::build_tab_bar_view;
use crate::ui::text_utils::truncate_to_width;
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

pub(crate) struct JumpWidget {
    _private: (),
}

impl JumpWidget {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl Widget for JumpWidget {
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
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        refresh_jump_rows(frame);
        clamp_overlay_tables(frame.view, frame.state, frame.jump_rows.len());
        draw_jump_layout(frame);
        draw_jump_table(frame);
        Ok(())
    }
}

fn refresh_jump_rows(frame: &mut WidgetFrame<'_, '_, '_, '_>) {
    frame.jump_rows.clear();
    if !frame.view.overlay.is(crate::ui::overlay::OverlayKind::Jump) {
        return;
    }
    let rows = frame
        .state
        .with_active_tab(|tab| {
            build_jump_rows(
                &tab.app.messages,
                frame.state.msg_width,
                max_preview_width(frame.state.msg_area),
                tab.app.pending_assistant,
            )
        })
        .unwrap_or_default();
    frame.jump_rows.extend(rows);
}

fn draw_jump_layout(frame: &mut WidgetFrame<'_, '_, '_, '_>) {
    let params = JumpLayoutParams {
        theme: frame.state.theme,
        tab_labels: frame.state.tab_labels,
        active_tab_pos: frame.state.active_tab_pos,
        categories: frame.state.categories,
        active_category: frame.state.active_category,
        header_note: frame.state.header_note,
        startup_text: frame.state.startup_text,
        header_area: frame.state.header_area,
        category_area: frame.state.category_area,
        tabs_area: frame.state.tabs_area,
        footer_area: frame.state.footer_area,
    };
    draw_jump_layout_base(frame.frame, params);
}

fn draw_jump_table(frame: &mut WidgetFrame<'_, '_, '_, '_>) {
    draw_jump_table_base(
        frame.frame,
        frame.state.msg_area,
        frame.jump_rows,
        frame.view.jump.selected,
        frame.state.theme,
        frame.view.jump.scroll,
    );
}

struct JumpLayoutParams<'a> {
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

fn draw_jump_layout_base(f: &mut ratatui::Frame<'_>, params: JumpLayoutParams<'_>) {
    draw_header(f, params.header_area, params.theme, params.header_note);
    draw_categories(
        f,
        params.category_area,
        params.categories,
        params.active_category,
        params.theme,
    );
    draw_tabs(
        f,
        params.tabs_area,
        params.tab_labels,
        params.active_tab_pos,
        params.theme,
        params.startup_text,
    );
    draw_footer(f, params.footer_area, params.theme, false, false);
}

fn draw_jump_table_base(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    rows: &[JumpRow],
    selected: usize,
    theme: &RenderTheme,
    scroll: usize,
) {
    let popup = build_jump_table(rows, selected, theme, scroll);
    draw_overlay_table(f, area, popup);
}

fn build_jump_table<'a>(
    rows: &'a [JumpRow],
    selected: usize,
    theme: &'a RenderTheme,
    scroll: usize,
) -> OverlayTable<'a> {
    OverlayTable {
        title: Line::from(jump_title()),
        header: jump_header(theme),
        rows: jump_body(rows),
        widths: jump_widths(),
        selected,
        scroll,
        theme,
    }
}

fn jump_header(theme: &RenderTheme) -> Row<'static> {
    Row::new(vec![
        Cell::from("序号"),
        Cell::from("角色"),
        Cell::from("内容"),
    ])
    .style(header_style(theme))
}

fn jump_body<'a>(rows: &'a [JumpRow]) -> Vec<Row<'a>> {
    rows.iter()
        .map(|row| {
            Row::new(vec![
                Cell::from(row.index.to_string()),
                Cell::from(row.role.clone()),
                Cell::from(row.preview.clone()),
            ])
        })
        .collect()
}

fn jump_widths() -> Vec<Constraint> {
    vec![
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Min(10),
    ]
}

fn jump_title() -> &'static str {
    "消息定位 · Enter/点击 跳转 · E 复制用户消息到新对话 · F2 退出"
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
