use crate::render::theme::RenderTheme;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
pub(crate) struct TableBuild {
    in_table: bool,
    in_head: bool,
    in_row: bool,
    pub(crate) in_cell: bool,
    streaming: bool,
    current_cell: String,
    current_row: Vec<String>,
    header: Option<Vec<String>>,
    rows: Vec<Vec<String>>,
}

impl TableBuild {
    pub(crate) fn start(&mut self, streaming: bool) {
        *self = Self {
            in_table: true,
            streaming,
            ..Default::default()
        };
    }

    pub(crate) fn start_head(&mut self) {
        if self.in_table {
            self.in_head = true;
        }
    }

    pub(crate) fn end_head(&mut self) {
        self.in_head = false;
    }

    pub(crate) fn start_row(&mut self) {
        if self.in_table {
            self.in_row = true;
            self.current_row.clear();
        }
    }

    pub(crate) fn end_row(&mut self) {
        if self.in_table && self.in_row {
            if self.in_head {
                self.header = Some(self.current_row.clone());
            } else if !self.current_row.is_empty() {
                self.rows.push(self.current_row.clone());
            }
            self.current_row.clear();
            self.in_row = false;
        }
    }

    pub(crate) fn start_cell(&mut self) {
        if self.in_table && self.in_row {
            self.in_cell = true;
            self.current_cell.clear();
        }
    }

    pub(crate) fn end_cell(&mut self) {
        if self.in_table && self.in_cell {
            self.current_row
                .push(self.current_cell.trim().to_string());
            self.current_cell.clear();
            self.in_cell = false;
        }
    }

    pub(crate) fn push_text(&mut self, text: &str) {
        if self.in_table && self.in_cell {
            self.current_cell.push_str(text);
        }
    }

    pub(crate) fn finish_render(
        &mut self,
        width: usize,
        theme: &RenderTheme,
    ) -> Vec<Line<'static>> {
        let mut out = Vec::new();
        if !self.in_table {
            return out;
        }
        let cols = table_cols(&self.header, &self.rows);
        if cols == 0 {
            self.in_table = false;
            return out;
        }
        let style = Style::default().fg(theme.fg.unwrap_or(Color::White));

        if self.streaming {
            if let Some(header) = &self.header {
                out.push(Line::from(Span::styled(
                    render_table_row_unaligned(header),
                    style.add_modifier(Modifier::BOLD),
                )));
                out.push(Line::from(Span::styled(
                    render_table_rule_unaligned(header),
                    style,
                )));
            }
            for row in &self.rows {
                out.push(Line::from(Span::styled(
                    render_table_row_unaligned(row),
                    style,
                )));
            }
            self.in_table = false;
            return out;
        }

        let widths = compute_table_widths(&self.header, &self.rows, cols, width);
        if let Some(header) = &self.header {
            out.push(Line::from(Span::styled(
                render_table_row(header, &widths),
                style.add_modifier(Modifier::BOLD),
            )));
            out.push(Line::from(Span::styled(
                render_table_rule(&widths),
                style,
            )));
        }
        for row in &self.rows {
            out.push(Line::from(Span::styled(
                render_table_row(row, &widths),
                style,
            )));
        }
        self.in_table = false;
        out
    }

    pub(crate) fn finish_count(&mut self, width: usize) -> usize {
        if !self.in_table {
            return 0;
        }
        let cols = table_cols(&self.header, &self.rows);
        if cols == 0 {
            self.in_table = false;
            return 0;
        }
        let _ = compute_table_widths(&self.header, &self.rows, cols, width);
        let header_lines = if self.header.is_some() { 2 } else { 0 };
        let row_lines = self.rows.len();
        self.in_table = false;
        header_lines + row_lines
    }
}

fn table_cols(header: &Option<Vec<String>>, rows: &[Vec<String>]) -> usize {
    header
        .as_ref()
        .map(|h| h.len())
        .unwrap_or(0)
        .max(rows.iter().map(|r| r.len()).max().unwrap_or(0))
}

fn compute_table_widths(
    header: &Option<Vec<String>>,
    rows: &[Vec<String>],
    cols: usize,
    width: usize,
) -> Vec<usize> {
    let mut widths = vec![1usize; cols];
    if let Some(h) = header {
        for (i, cell) in h.iter().enumerate() {
            let w = UnicodeWidthStr::width(cell.as_str());
            if w > widths[i] {
                widths[i] = w;
            }
        }
    }
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            let w = UnicodeWidthStr::width(cell.as_str());
            if w > widths[i] {
                widths[i] = w;
            }
        }
    }
    let total_sep = cols + 1;
    let total_content: usize = widths.iter().sum();
    let total = total_sep + total_content + cols * 2;
    if total > width.max(10) {
        // Keep widths; table may wrap but column alignment still uses content width.
    }
    widths
}

fn render_table_row(row: &[String], widths: &[usize]) -> String {
    let mut out = String::new();
    out.push('|');
    for (i, w) in widths.iter().enumerate() {
        let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
        let cell_width = UnicodeWidthStr::width(cell);
        out.push(' ');
        out.push_str(cell);
        if *w > cell_width {
            out.push_str(&" ".repeat(*w - cell_width));
        }
        out.push(' ');
        out.push('|');
    }
    out
}

fn render_table_rule(widths: &[usize]) -> String {
    let mut out = String::new();
    out.push('|');
    for w in widths {
        out.push(' ');
        out.push_str(&"-".repeat(*w));
        out.push(' ');
        out.push('|');
    }
    out
}

fn render_table_row_unaligned(row: &[String]) -> String {
    let mut out = String::new();
    out.push('|');
    for cell in row {
        out.push(' ');
        out.push_str(cell);
        out.push(' ');
        out.push('|');
    }
    out
}

fn render_table_rule_unaligned(header: &[String]) -> String {
    let mut out = String::new();
    out.push('|');
    for cell in header {
        let w = UnicodeWidthStr::width(cell.as_str()).max(3);
        out.push(' ');
        out.push_str(&"-".repeat(w));
        out.push(' ');
        out.push('|');
    }
    out
}
