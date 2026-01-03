use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Copy, Clone, Debug)]
pub struct Selection {
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl Selection {
    pub fn ordered(self) -> ((usize, usize), (usize, usize)) {
        if self.start.0 < self.end.0 {
            (self.start, self.end)
        } else if self.start.0 > self.end.0 {
            (self.end, self.start)
        } else if self.start.1 <= self.end.1 {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

pub fn line_to_string(line: &Line<'_>) -> String {
    let mut out = String::new();
    for span in &line.spans {
        out.push_str(span.content.as_ref());
    }
    out
}

pub fn line_width(line: &Line<'_>) -> usize {
    line_to_string(line).width()
}

pub fn col_from_x(line: &Line<'_>, x: usize) -> usize {
    line_width(line).min(x)
}

pub fn apply_selection_to_text(
    text: &Text<'_>,
    scroll: usize,
    selection: Selection,
    select_style: Style,
) -> Text<'static> {
    let ((start_line, start_col), (end_line, end_col)) = selection.ordered();
    let mut out: Vec<Line<'static>> = Vec::with_capacity(text.lines.len());
    for (idx, line) in text.lines.iter().enumerate() {
        let global_line = scroll + idx;
        let Some((sel_start, sel_end)) = selection_range_for_line(
            global_line,
            start_line,
            end_line,
            start_col,
            end_col,
            line_width(line),
        ) else {
            out.push(to_owned_line(line));
            continue;
        };
        out.push(apply_selection_to_line(
            line,
            sel_start,
            sel_end,
            select_style,
        ));
    }
    Text::from(out)
}

fn selection_range_for_line(
    global_line: usize,
    start_line: usize,
    end_line: usize,
    start_col: usize,
    end_col: usize,
    line_len: usize,
) -> Option<(usize, usize)> {
    if global_line < start_line || global_line > end_line {
        return None;
    }
    let (sel_start, sel_end) = if start_line == end_line {
        (start_col.min(line_len), end_col.min(line_len))
    } else if global_line == start_line {
        (start_col.min(line_len), line_len)
    } else if global_line == end_line {
        (0, end_col.min(line_len))
    } else {
        (0, line_len)
    };
    if sel_start == sel_end {
        return None;
    }
    Some((sel_start, sel_end))
}

fn apply_selection_to_line(
    line: &Line<'_>,
    sel_start: usize,
    sel_end: usize,
    select_style: Style,
) -> Line<'static> {
    let spans = build_selected_spans(line, sel_start, sel_end, select_style);
    Line {
        style: line.style,
        alignment: line.alignment,
        spans,
    }
}

fn build_selected_spans(
    line: &Line<'_>,
    sel_start: usize,
    sel_end: usize,
    select_style: Style,
) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut col = 0usize;
    let mut current_style: Option<Style> = None;
    let mut buffer = String::new();
    for span in &line.spans {
        process_span(
            span,
            ProcessSpanParams {
                sel_start,
                sel_end,
                select_style,
            },
            &mut col,
            &mut spans,
            &mut current_style,
            &mut buffer,
        );
    }
    flush_buffer(&mut spans, current_style, &mut buffer);
    spans
}

struct ProcessSpanParams {
    sel_start: usize,
    sel_end: usize,
    select_style: Style,
}

fn process_span(
    span: &Span<'_>,
    params: ProcessSpanParams,
    col: &mut usize,
    spans: &mut Vec<Span<'static>>,
    current_style: &mut Option<Style>,
    buffer: &mut String,
) {
    let base_style = span.style;
    for ch in span.content.chars() {
        let width = UnicodeWidthChar::width(ch).unwrap_or(0).max(1);
        let next = col.saturating_add(width);
        let selected = next > params.sel_start && *col < params.sel_end;
        let style = if selected {
            base_style.patch(params.select_style)
        } else {
            base_style
        };
        update_buffer(spans, current_style, buffer, style, ch);
        *col = next;
    }
}

fn update_buffer(
    spans: &mut Vec<Span<'static>>,
    current_style: &mut Option<Style>,
    buffer: &mut String,
    style: Style,
    ch: char,
) {
    if current_style.map(|s| s != style).unwrap_or(true) {
        flush_buffer(spans, *current_style, buffer);
        *current_style = Some(style);
    }
    buffer.push(ch);
}

fn flush_buffer(spans: &mut Vec<Span<'static>>, style: Option<Style>, buffer: &mut String) {
    if let Some(style) = style
        && !buffer.is_empty()
    {
        spans.push(Span::styled(std::mem::take(buffer), style));
    }
}

fn to_owned_line(line: &Line<'_>) -> Line<'static> {
    let spans = line
        .spans
        .iter()
        .map(|span| Span::styled(span.content.to_string(), span.style))
        .collect();
    Line {
        style: line.style,
        alignment: line.alignment,
        spans,
    }
}

pub fn slice_line_by_cols(line: &str, start_col: usize, end_col: usize) -> String {
    if start_col >= end_col {
        return String::new();
    }
    let mut out = String::new();
    let mut col = 0usize;
    for ch in line.chars() {
        let width = UnicodeWidthChar::width(ch).unwrap_or(0).max(1);
        let next = col.saturating_add(width);
        if next > start_col && col < end_col {
            out.push(ch);
        }
        col = next;
    }
    out
}

pub fn extract_selection(lines: &[String], selection: Selection) -> String {
    let ((start_line, start_col), (end_line, end_col)) = selection.ordered();
    if start_line >= lines.len() {
        return String::new();
    }
    let end_line = end_line.min(lines.len().saturating_sub(1));
    let mut out = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let Some((sel_start, sel_end)) =
            selection_range_for_line(idx, start_line, end_line, start_col, end_col, line.width())
        else {
            continue;
        };
        out.push(slice_line_by_cols(line, sel_start, sel_end));
    }
    out.join("\n")
}

pub fn chat_position_from_mouse(
    text: &Text<'static>,
    scroll: u16,
    inner: Rect,
    mouse_x: u16,
    mouse_y: u16,
) -> (usize, usize) {
    let local_y = mouse_y.saturating_sub(inner.y);
    let row = (scroll as usize).saturating_add(local_y as usize);
    let col_x = mouse_x.saturating_sub(inner.x) as usize;
    let line = text.lines.get(local_y as usize);
    let col = line.map(|l| col_from_x(l, col_x)).unwrap_or(0);
    (row, col)
}
