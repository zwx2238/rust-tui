use crate::render::RenderTheme;
use crate::ui::draw::{
    draw_categories, draw_footer, draw_header, draw_tabs, inner_area, layout_chunks,
};
use crate::ui::runtime_layout::compute_sidebar_width;
use ratatui::layout::Rect;

pub(crate) struct SummaryLayout {
    pub(crate) header_area: Rect,
    pub(crate) category_area: Rect,
    pub(crate) tabs_area: Rect,
    pub(crate) body_area: Rect,
    pub(crate) footer_area: Rect,
    pub(crate) max_latest_width: usize,
}

pub(crate) struct DrawSummaryLayoutParams<'a, 'b> {
    pub(crate) f: &'a mut ratatui::Frame<'b>,
    pub(crate) theme: &'a RenderTheme,
    pub(crate) tab_labels: &'a [String],
    pub(crate) active_tab_pos: usize,
    pub(crate) categories: &'a [String],
    pub(crate) active_category: usize,
    pub(crate) header_note: Option<&'a str>,
    pub(crate) startup_text: Option<&'a str>,
    pub(crate) header_area: Rect,
    pub(crate) category_area: Rect,
    pub(crate) tabs_area: Rect,
    pub(crate) footer_area: Rect,
}

pub(crate) fn build_summary_layout(
    size: ratatui::layout::Size,
    categories: &[String],
) -> SummaryLayout {
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

pub(crate) fn draw_summary_layout<'a, 'b>(params: DrawSummaryLayoutParams<'a, 'b>) {
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
