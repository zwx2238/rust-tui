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

pub fn redraw(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    tab_labels: &[String],
    active_tab_pos: usize,
    categories: &[String],
    active_category: usize,
    startup_text: Option<&str>,
    input_height: u16,
    header_note: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let size = terminal.size()?;
    let size = Rect::new(0, 0, size.width, size.height);
    let sidebar_width = crate::ui::runtime_layout::compute_sidebar_width(categories, size.width);
    let (header_area, category_area, tabs_area, msg_area, input_area, footer_area) =
        layout_chunks(size, input_height, sidebar_width);
    terminal.draw(|f| {
        draw_base(
            f,
            app,
            theme,
            text,
            total_lines,
            header_area,
            category_area,
            tabs_area,
            msg_area,
            input_area,
            footer_area,
            tab_labels,
            active_tab_pos,
            categories,
            active_category,
            startup_text,
            header_note,
        );
        draw_notice(f, size, app, theme);
    })?;
    Ok(())
}

pub fn redraw_with_overlay<F>(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    tab_labels: &[String],
    active_tab_pos: usize,
    categories: &[String],
    active_category: usize,
    startup_text: Option<&str>,
    input_height: u16,
    overlay: F,
    header_note: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(&mut ratatui::Frame<'_>),
{
    let size = terminal.size()?;
    let size = Rect::new(0, 0, size.width, size.height);
    let sidebar_width = crate::ui::runtime_layout::compute_sidebar_width(categories, size.width);
    let (header_area, category_area, tabs_area, msg_area, input_area, footer_area) =
        layout_chunks(size, input_height, sidebar_width);
    terminal.draw(|f| {
        draw_base(
            f,
            app,
            theme,
            text,
            total_lines,
            header_area,
            category_area,
            tabs_area,
            msg_area,
            input_area,
            footer_area,
            tab_labels,
            active_tab_pos,
            categories,
            active_category,
            startup_text,
            header_note,
        );
        overlay(f);
        draw_notice(f, size, app, theme);
    })?;
    Ok(())
}

pub(crate) use tabs::draw_tabs;

fn draw_base(
    f: &mut ratatui::Frame<'_>,
    app: &mut App,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    header_area: Rect,
    category_area: Rect,
    tabs_area: Rect,
    msg_area: Rect,
    input_area: Rect,
    footer_area: Rect,
    tab_labels: &[String],
    active_tab_pos: usize,
    categories: &[String],
    active_category: usize,
    startup_text: Option<&str>,
    header_note: Option<&str>,
) {
    header_footer::draw_header(f, header_area, theme, header_note);
    categories::draw_categories(f, category_area, categories, active_category, theme);
    tabs::draw_tabs(
        f,
        tabs_area,
        tab_labels,
        active_tab_pos,
        theme,
        startup_text,
    );
    messages::draw_messages(
        f,
        msg_area,
        text,
        app.scroll,
        theme,
        app.focus == Focus::Chat,
        total_lines,
        app.chat_selection,
    );
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
    draw_command_suggestions(f, msg_area, input_area, app, theme);
    header_footer::draw_footer(f, footer_area, theme, app.nav_mode);
}
