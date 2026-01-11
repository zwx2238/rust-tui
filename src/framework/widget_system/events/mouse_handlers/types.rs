use crate::render::RenderTheme;
use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crossterm::event::MouseEvent;
use ratatui::layout::Rect;

pub(crate) struct MouseEventParams<'a> {
    pub m: MouseEvent,
    pub tabs: &'a mut [TabState],
    pub active_tab: &'a mut usize,
    pub categories: &'a [String],
    pub active_category: &'a mut usize,
    pub tabs_area: Rect,
    pub msg_area: Rect,
    pub input_area: Rect,
    pub category_area: Rect,
    pub msg_width: usize,
    pub view_height: u16,
    pub total_lines: usize,
    pub theme: &'a RenderTheme,
}

pub(crate) struct MouseDownParams<'a> {
    pub m: MouseEvent,
    pub tabs: &'a mut [TabState],
    pub active_tab: &'a mut usize,
    pub categories: &'a [String],
    pub active_category: &'a mut usize,
    pub tabs_area: Rect,
    pub msg_area: Rect,
    pub input_area: Rect,
    pub category_area: Rect,
    pub msg_width: usize,
    pub view_height: u16,
    pub total_lines: usize,
    pub theme: &'a RenderTheme,
}

pub(crate) struct MouseDragParams<'a> {
    pub m: MouseEvent,
    pub tabs: &'a mut [TabState],
    pub active_tab: usize,
    pub msg_area: Rect,
    pub input_area: Rect,
    pub msg_width: usize,
    pub view_height: u16,
    pub total_lines: usize,
    pub theme: &'a RenderTheme,
}
