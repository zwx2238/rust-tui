#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::runtime_events::handle_mouse_event;
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::selection::Selection;
    use crate::ui::state::Focus;
    use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    struct MouseCtx {
        tabs: Vec<TabState>,
        active_tab: usize,
        active_category: usize,
        categories: Vec<String>,
        tabs_area: Rect,
        msg_area: Rect,
        input_area: Rect,
        category_area: Rect,
    }

    fn theme() -> RenderTheme {
        RenderTheme {
            bg: Color::Black,
            fg: Some(Color::White),
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
    }

    fn base_ctx() -> MouseCtx {
        MouseCtx {
            tabs: vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")],
            active_tab: 0,
            active_category: 0,
            categories: vec!["默认".to_string()],
            msg_area: Rect::new(0, 1, 40, 10),
            input_area: Rect::new(0, 11, 40, 3),
            tabs_area: Rect::new(200, 200, 0, 0),
            category_area: Rect::new(200, 200, 0, 0),
        }
    }

    fn handle_mouse(ctx: &mut MouseCtx, event: MouseEvent) -> Option<usize> {
        handle_mouse_event(
            event,
            &mut ctx.tabs,
            &mut ctx.active_tab,
            &ctx.categories,
            &mut ctx.active_category,
            ctx.tabs_area,
            ctx.msg_area,
            ctx.input_area,
            ctx.category_area,
            40,
            5,
            20,
            &theme(),
        )
    }

    #[test]
    fn mouse_down_on_message_starts_chat_selection() {
        let mut ctx = base_ctx();
        ctx.tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        let m = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: ctx.msg_area.x + 2,
            row: ctx.msg_area.y + 2,
            modifiers: KeyModifiers::NONE,
        };
        let _ = handle_mouse(&mut ctx, m);
        assert_eq!(ctx.tabs[0].app.focus, Focus::Chat);
        assert!(ctx.tabs[0].app.chat_selecting);
        assert!(ctx.tabs[0].app.chat_selection.is_some());
    }

    #[test]
    fn mouse_drag_updates_chat_selection() {
        let mut ctx = base_ctx();
        ctx.tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello world".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        ctx.tabs[0].app.chat_selecting = true;
        ctx.tabs[0].app.chat_selection = Some(Selection { start: (0, 0), end: (0, 0) });
        let m = MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: ctx.msg_area.x + 5,
            row: ctx.msg_area.y + 2,
            modifiers: KeyModifiers::NONE,
        };
        let _ = handle_mouse(&mut ctx, m);
        let sel = ctx.tabs[0].app.chat_selection.unwrap();
        assert!(sel.end.1 >= sel.start.1);
    }

    #[test]
    fn mouse_up_clears_empty_selection() {
        let mut ctx = base_ctx();
        ctx.tabs[0].app.chat_selecting = true;
        ctx.tabs[0].app.chat_selection = Some(Selection { start: (0, 0), end: (0, 0) });
        let m = MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: ctx.msg_area.x + 1,
            row: ctx.msg_area.y + 1,
            modifiers: KeyModifiers::NONE,
        };
        let _ = handle_mouse(&mut ctx, m);
        assert!(!ctx.tabs[0].app.chat_selecting);
        assert!(ctx.tabs[0].app.chat_selection.is_none());
    }

    #[test]
    fn mouse_down_on_edit_button_returns_index() {
        let mut ctx = base_ctx();
        ctx.tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        ctx.tabs[0].app.message_layouts = vec![crate::render::MessageLayout {
            index: 3,
            label_line: 0,
            button_range: Some((0, 4)),
        }];
        let m = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: ctx.msg_area.x + 2,
            row: ctx.msg_area.y + 1,
            modifiers: KeyModifiers::NONE,
        };
        let hit = handle_mouse(&mut ctx, m);
        assert_eq!(hit, Some(3));
    }
}
