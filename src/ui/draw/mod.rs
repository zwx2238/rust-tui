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
#[allow(unused_imports)]
pub(crate) use messages::draw_messages;
pub use layout::{
    inner_area, inner_height, inner_width, input_inner_area, layout_chunks, scrollbar_area,
};

pub fn redraw(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App, theme: &RenderTheme,
    text: &Text<'_>, total_lines: usize, tab_labels: &[String], active_tab_pos: usize,
    categories: &[String], active_category: usize, startup_text: Option<&str>,
    input_height: u16, header_note: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let areas = compute_draw_areas(terminal, categories, input_height)?;
    terminal.draw(|f| {
        draw_base_frame(
            f,
            app,
            theme,
            text,
            total_lines,
            &areas,
            tab_labels,
            active_tab_pos,
            categories,
            active_category,
            startup_text,
            header_note,
        );
        draw_notice(f, areas.size, app, theme);
    })?;
    Ok(())
}

pub fn redraw_with_overlay<F>(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App, theme: &RenderTheme,
    text: &Text<'_>, total_lines: usize, tab_labels: &[String], active_tab_pos: usize,
    categories: &[String], active_category: usize, startup_text: Option<&str>,
    input_height: u16, overlay: F, header_note: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(&mut ratatui::Frame<'_>),
{
    let areas = compute_draw_areas(terminal, categories, input_height)?;
    terminal.draw(|f| {
        draw_base_frame(
            f,
            app,
            theme,
            text,
            total_lines,
            &areas,
            tab_labels,
            active_tab_pos,
            categories,
            active_category,
            startup_text,
            header_note,
        );
        overlay(f);
        draw_notice(f, areas.size, app, theme);
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

fn draw_base_frame(
    f: &mut ratatui::Frame<'_>,
    app: &mut App,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    areas: &DrawAreas,
    tab_labels: &[String],
    active_tab_pos: usize,
    categories: &[String],
    active_category: usize,
    startup_text: Option<&str>,
    header_note: Option<&str>,
) {
    draw_base_header(
        f,
        theme,
        areas,
        tab_labels,
        active_tab_pos,
        categories,
        active_category,
        startup_text,
        header_note,
    );
    draw_base_messages(f, app, theme, text, total_lines, areas);
    draw_input_area(f, app, theme, areas.input_area);
    draw_command_suggestions(f, areas.msg_area, areas.input_area, app, theme);
    header_footer::draw_footer(f, areas.footer_area, theme, app.nav_mode);
}

fn draw_base_header(
    f: &mut ratatui::Frame<'_>,
    theme: &RenderTheme,
    areas: &DrawAreas,
    tab_labels: &[String],
    active_tab_pos: usize,
    categories: &[String],
    active_category: usize,
    startup_text: Option<&str>,
    header_note: Option<&str>,
) {
    header_footer::draw_header(f, areas.header_area, theme, header_note);
    categories::draw_categories(f, areas.category_area, categories, active_category, theme);
    tabs::draw_tabs(
        f,
        areas.tabs_area,
        tab_labels,
        active_tab_pos,
        theme,
        startup_text,
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
        areas.msg_area,
        text,
        app.scroll,
        theme,
        app.focus == Focus::Chat,
        total_lines,
        app.chat_selection,
    );
}

fn draw_input_area(f: &mut ratatui::Frame<'_>, app: &mut App, theme: &RenderTheme, input_area: Rect) {
    let input_disabled =
        app.busy || app.pending_code_exec.is_some() || app.pending_file_patch.is_some();
    crate::ui::draw_input::draw_input(
        f,
        input_area,
        &mut app.input,
        theme,
        app.focus == Focus::Input && !input_disabled,
        input_disabled,
        &app.model_key,
        &app.prompt_key,
    );
}
