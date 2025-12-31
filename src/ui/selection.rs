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
        if global_line < start_line || global_line > end_line {
            out.push(to_owned_line(line));
            continue;
        }
        let line_len = line_width(line);
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
            out.push(to_owned_line(line));
            continue;
        }
        out.push(apply_selection_to_line(line, sel_start, sel_end, select_style));
    }
    Text::from(out)
}

fn apply_selection_to_line(
    line: &Line<'_>,
    sel_start: usize,
    sel_end: usize,
    select_style: Style,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut col = 0usize;
    let mut current_style: Option<Style> = None;
    let mut buffer = String::new();
    let flush = |spans: &mut Vec<Span<'static>>,
                 style: Option<Style>,
                 buffer: &mut String| {
        if let Some(style) = style {
            if !buffer.is_empty() {
                spans.push(Span::styled(std::mem::take(buffer), style));
            }
        }
    };

    for span in &line.spans {
        let base_style = span.style;
        for ch in span.content.chars() {
            let width = UnicodeWidthChar::width(ch).unwrap_or(0).max(1);
            let next = col.saturating_add(width);
            let selected = next > sel_start && col < sel_end;
            let style = if selected {
                base_style.patch(select_style)
            } else {
                base_style
            };
            if current_style.map(|s| s != style).unwrap_or(true) {
                flush(&mut spans, current_style, &mut buffer);
                current_style = Some(style);
            }
            buffer.push(ch);
            col = next;
        }
    }
    flush(&mut spans, current_style, &mut buffer);

    Line {
        style: line.style,
        alignment: line.alignment,
        spans,
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
        if idx < start_line || idx > end_line {
            continue;
        }
        let line_width = line.width();
        let (sel_start, sel_end) = if start_line == end_line {
            (start_col.min(line_width), end_col.min(line_width))
        } else if idx == start_line {
            (start_col.min(line_width), line_width)
        } else if idx == end_line {
            (0, end_col.min(line_width))
        } else {
            (0, line_width)
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
