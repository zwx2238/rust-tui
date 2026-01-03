use crate::render::RenderTheme;
use crate::ui::command_suggestions::draw_command_suggestions;
use crate::ui::notice::draw_notice;
use crate::ui::state::{App, Focus};
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::Stdout;

mod categories;
mod header_footer;
pub mod layout;
mod messages;
pub(crate) mod style;
mod tabs;

pub(crate) use categories::draw_categories;
pub(crate) use header_footer::{draw_footer, draw_header};
pub use layout::{
    inner_area, inner_height, inner_width, input_inner_area, layout_chunks, scrollbar_area,
};
#[allow(unused_imports)]
pub(crate) use messages::{MessagesDrawParams, draw_messages};

pub struct RedrawParams<'a> {
    pub terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    pub app: &'a mut App,
    pub theme: &'a RenderTheme,
    pub text: &'a Text<'a>,
    pub total_lines: usize,
    pub tab_labels: &'a [String],
    pub active_tab_pos: usize,
    pub categories: &'a [String],
    pub active_category: usize,
    pub startup_text: Option<&'a str>,
    pub input_height: u16,
    pub header_note: Option<&'a str>,
}

pub fn redraw(params: RedrawParams<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let areas = compute_draw_areas(params.terminal, params.categories, params.input_height)?;
    params.terminal.draw(|f| {
        draw_base_frame(
            f,
            BaseFrameParams {
                app: params.app,
                theme: params.theme,
                text: params.text,
                total_lines: params.total_lines,
                tab_labels: params.tab_labels,
                active_tab_pos: params.active_tab_pos,
                categories: params.categories,
                active_category: params.active_category,
                startup_text: params.startup_text,
                header_note: params.header_note,
            },
            &areas,
        );
        draw_notice(f, areas.size, params.app, params.theme);
    })?;
    Ok(())
}

pub struct RedrawWithOverlayParams<'a> {
    pub terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    pub app: &'a mut App,
    pub theme: &'a RenderTheme,
    pub text: &'a Text<'a>,
    pub total_lines: usize,
    pub tab_labels: &'a [String],
    pub active_tab_pos: usize,
    pub categories: &'a [String],
    pub active_category: usize,
    pub startup_text: Option<&'a str>,
    pub input_height: u16,
    pub header_note: Option<&'a str>,
}

pub fn redraw_with_overlay<F>(
    params: RedrawWithOverlayParams<'_>,
    overlay: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(&mut ratatui::Frame<'_>),
{
    let areas = compute_draw_areas(params.terminal, params.categories, params.input_height)?;
    params.terminal.draw(|f| {
        draw_base_frame(
            f,
            BaseFrameParams {
                app: params.app,
                theme: params.theme,
                text: params.text,
                total_lines: params.total_lines,
                tab_labels: params.tab_labels,
                active_tab_pos: params.active_tab_pos,
                categories: params.categories,
                active_category: params.active_category,
                startup_text: params.startup_text,
                header_note: params.header_note,
            },
            &areas,
        );
        overlay(f);
        draw_notice(f, areas.size, params.app, params.theme);
    })?;
    Ok(())
}

pub(crate) use tabs::draw_tabs;

struct DrawAreas {
    size: Rect,
    header_area: Rect,
    category_area: Rect,
    tabs_area: Rect,
    msg_area: Rect,
    input_area: Rect,
    footer_area: Rect,
}

fn compute_draw_areas(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    categories: &[String],
    input_height: u16,
) -> Result<DrawAreas, Box<dyn std::error::Error>> {
    let size = terminal.size()?;
    let size = Rect::new(0, 0, size.width, size.height);
    let sidebar_width = crate::ui::runtime_layout::compute_sidebar_width(categories, size.width);
    let (header_area, category_area, tabs_area, msg_area, input_area, footer_area) =
        layout_chunks(size, input_height, sidebar_width);
    Ok(DrawAreas {
        size,
        header_area,
        category_area,
        tabs_area,
        msg_area,
        input_area,
        footer_area,
    })
}

struct BaseFrameParams<'a> {
    app: &'a mut App,
    theme: &'a RenderTheme,
    text: &'a Text<'a>,
    total_lines: usize,
    tab_labels: &'a [String],
    active_tab_pos: usize,
    categories: &'a [String],
    active_category: usize,
    startup_text: Option<&'a str>,
    header_note: Option<&'a str>,
}

fn draw_base_frame(f: &mut ratatui::Frame<'_>, params: BaseFrameParams<'_>, areas: &DrawAreas) {
    draw_base_header(
        f,
        BaseHeaderParams {
            theme: params.theme,
            tab_labels: params.tab_labels,
            active_tab_pos: params.active_tab_pos,
            categories: params.categories,
            active_category: params.active_category,
            startup_text: params.startup_text,
            header_note: params.header_note,
        },
        areas,
    );
    draw_base_messages(
        f,
        params.app,
        params.theme,
        params.text,
        params.total_lines,
        areas,
    );
    draw_input_area(f, params.app, params.theme, areas.input_area);
    draw_command_suggestions(
        f,
        areas.msg_area,
        areas.input_area,
        params.app,
        params.theme,
    );
    header_footer::draw_footer(f, areas.footer_area, params.theme, params.app.nav_mode);
}

struct BaseHeaderParams<'a> {
    theme: &'a RenderTheme,
    tab_labels: &'a [String],
    active_tab_pos: usize,
    categories: &'a [String],
    active_category: usize,
    startup_text: Option<&'a str>,
    header_note: Option<&'a str>,
}

fn draw_base_header(f: &mut ratatui::Frame<'_>, params: BaseHeaderParams<'_>, areas: &DrawAreas) {
    header_footer::draw_header(f, areas.header_area, params.theme, params.header_note);
    categories::draw_categories(
        f,
        areas.category_area,
        params.categories,
        params.active_category,
        params.theme,
    );
    tabs::draw_tabs(
        f,
        areas.tabs_area,
        params.tab_labels,
        params.active_tab_pos,
        params.theme,
        params.startup_text,
    );
}

fn draw_base_messages(
    f: &mut ratatui::Frame<'_>,
    app: &App,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    areas: &DrawAreas,
) {
    messages::draw_messages(
        f,
        messages::MessagesDrawParams {
            area: areas.msg_area,
            text,
            scroll: app.scroll,
            theme,
            focused: app.focus == Focus::Chat,
            total_lines,
            selection: app.chat_selection,
        },
    );
}

fn draw_input_area(
    f: &mut ratatui::Frame<'_>,
    app: &mut App,
    theme: &RenderTheme,
    input_area: Rect,
) {
    let input_disabled =
        app.busy || app.pending_code_exec.is_some() || app.pending_file_patch.is_some();
    crate::ui::draw_input::draw_input(
        f,
        crate::ui::draw_input::InputDrawParams {
            area: input_area,
            input: &mut app.input,
            theme,
            focused: app.focus == Focus::Input && !input_disabled,
            busy: input_disabled,
            model_key: &app.model_key,
            prompt_key: &app.prompt_key,
        },
    );
}
