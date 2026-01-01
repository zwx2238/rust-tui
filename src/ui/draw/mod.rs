use crate::render::RenderTheme;
use crate::ui::state::{App, Focus};
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;

pub mod layout;
mod messages;
pub(crate) mod style;
mod tabs;

pub use layout::{
    inner_area, inner_height, inner_width, input_inner_area, layout_chunks, scrollbar_area,
};

pub fn redraw(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    tabs_len: usize,
    active_tab: usize,
    startup_text: Option<&str>,
    input_height: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let size = terminal.size()?;
    let size = Rect::new(0, 0, size.width, size.height);
    let (tabs_area, msg_area, input_area) = layout_chunks(size, input_height);
    terminal.draw(|f| {
        draw_base(
            f,
            app,
            theme,
            text,
            total_lines,
            tabs_area,
            msg_area,
            input_area,
            tabs_len,
            active_tab,
            startup_text,
        );
    })?;
    Ok(())
}

pub fn redraw_with_overlay<F>(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    theme: &RenderTheme,
    text: &Text<'_>,
    total_lines: usize,
    tabs_len: usize,
    active_tab: usize,
    startup_text: Option<&str>,
    input_height: u16,
    overlay: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(&mut ratatui::Frame<'_>),
{
    let size = terminal.size()?;
    let size = Rect::new(0, 0, size.width, size.height);
    let (tabs_area, msg_area, input_area) = layout_chunks(size, input_height);
    terminal.draw(|f| {
        draw_base(
            f,
            app,
            theme,
            text,
            total_lines,
            tabs_area,
            msg_area,
            input_area,
            tabs_len,
            active_tab,
            startup_text,
        );
        overlay(f);
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
    tabs_area: Rect,
    msg_area: Rect,
    input_area: Rect,
    tabs_len: usize,
    active_tab: usize,
    startup_text: Option<&str>,
) {
    tabs::draw_tabs(f, tabs_area, tabs_len, active_tab, theme, startup_text);
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
    crate::ui::draw_input::draw_input(
        f,
        input_area,
        &mut app.input,
        theme,
        app.focus == Focus::Input,
        app.busy,
        &app.model_key,
        &app.prompt_key,
    );
}
