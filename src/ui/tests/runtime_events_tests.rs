#[cfg(test)]
mod tests {
    use crate::ui::runtime_events::{handle_paste_event, handle_tab_category_click};
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::state::{Focus, PendingCommand};
    use crate::ui::tab_bar::{TabBarItemKind, build_tab_bar_view};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
    use ratatui::layout::Rect;
    use unicode_width::UnicodeWidthStr;

    struct MouseCtx {
        tabs: Vec<TabState>,
        active_tab: usize,
        active_category: usize,
        categories: Vec<String>,
        msg_area: Rect,
        input_area: Rect,
        tabs_area: Rect,
        category_area: Rect,
        theme: crate::render::RenderTheme,
    }

    fn theme() -> crate::render::RenderTheme {
        crate::render::RenderTheme {
            bg: ratatui::style::Color::Black,
            fg: Some(ratatui::style::Color::White),
            code_bg: ratatui::style::Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(ratatui::style::Color::Cyan),
        }
    }

    fn base_mouse_ctx() -> MouseCtx {
        MouseCtx {
            tabs: vec![TabState::new(
                "id1".into(),
                "默认".into(),
                "",
                false,
                "m1",
                "p1",
            )],
            active_tab: 0,
            active_category: 0,
            categories: vec!["默认".to_string()],
            msg_area: Rect::new(0, 0, 40, 10),
            input_area: Rect::new(0, 10, 40, 3),
            tabs_area: Rect::new(0, 0, 40, 1),
            category_area: Rect::new(0, 0, 10, 5),
            theme: theme(),
        }
    }

    fn handle_mouse(ctx: &mut MouseCtx, event: MouseEvent) {
        crate::ui::runtime_events::handle_mouse_event(
            crate::ui::runtime_events::MouseEventParams {
                m: event,
                tabs: &mut ctx.tabs,
                active_tab: &mut ctx.active_tab,
                categories: &ctx.categories,
                active_category: &mut ctx.active_category,
                tabs_area: ctx.tabs_area,
                msg_area: ctx.msg_area,
                input_area: ctx.input_area,
                category_area: ctx.category_area,
                msg_width: 40,
                view_height: 10,
                total_lines: 100,
                theme: &ctx.theme,
            },
        );
    }

    #[test]
    fn handle_paste_event_inserts_text() {
        let mut tabs = vec![TabState::new(
            "id".into(),
            "默认".into(),
            "",
            false,
            "m1",
            "p1",
        )];
        tabs[0].app.focus = Focus::Input;
        handle_paste_event("a\r\nb", &mut tabs, 0);
        let text = tabs[0].app.input.lines().join("\n");
        assert_eq!(text, "a\nb");
    }

    #[test]
    fn handle_tab_category_click_updates_category() {
        let mut tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "分类 2".into(), "", false, "m1", "p1"),
        ];
        let mut active_tab = 0usize;
        let categories = vec!["默认".to_string(), "分类 2".to_string()];
        let mut active_category = 0usize;
        let tabs_area = Rect::new(0, 0, 20, 1);
        let category_area = Rect::new(0, 2, 10, 5);
        let handled =
            handle_tab_category_click(crate::ui::runtime_events::TabCategoryClickParams {
                mouse_x: 1,
                mouse_y: 3,
                tabs: &mut tabs,
                active_tab: &mut active_tab,
                categories: &categories,
                active_category: &mut active_category,
                tabs_area,
                category_area,
            });
        assert!(handled);
        assert_eq!(active_category, 1);
        assert_eq!(active_tab, 1);
    }

    #[test]
    fn handle_tab_category_click_updates_tab() {
        let mut tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "默认".into(), "", false, "m1", "p1"),
        ];
        let mut active_tab = 0usize;
        let categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let labels = crate::ui::runtime_helpers::tab_labels_for_category(&tabs, "默认");
        let tabs_area = Rect::new(0, 0, 20, 1);
        let category_area = Rect::new(0, 2, 10, 5);
        let second_tab_x = labels[0].width() as u16 + 1;
        let handled =
            handle_tab_category_click(crate::ui::runtime_events::TabCategoryClickParams {
                mouse_x: second_tab_x,
                mouse_y: 0,
                tabs: &mut tabs,
                active_tab: &mut active_tab,
                categories: &categories,
                active_category: &mut active_category,
                tabs_area,
                category_area,
            });
        assert!(handled);
        assert_eq!(active_tab, 1);
    }

    #[test]
    fn handle_tab_category_click_overflow_more_right_jumps() {
        let mut tabs = (0..10)
            .map(|i| TabState::new(format!("id{i}"), "默认".into(), "", false, "m1", "p1"))
            .collect::<Vec<_>>();
        let mut active_tab = 0usize;
        let categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let tabs_area = Rect::new(0, 0, 20, 1);
        let category_area = Rect::new(0, 2, 10, 5);

        let labels = crate::ui::runtime_helpers::tab_labels_for_category(&tabs, "默认");
        let visible = crate::ui::runtime_helpers::visible_tab_indices(&tabs, "默认");
        let active_pos = crate::ui::runtime_helpers::active_tab_position(&tabs, "默认", active_tab);
        let view = build_tab_bar_view(&labels, active_pos, tabs_area.width);

        let (target_pos, more_right_x) = more_right_target_and_x(tabs_area, &view);
        let expected = visible[target_pos];

        let handled =
            handle_tab_category_click(crate::ui::runtime_events::TabCategoryClickParams {
                mouse_x: more_right_x,
                mouse_y: 0,
                tabs: &mut tabs,
                active_tab: &mut active_tab,
                categories: &categories,
                active_category: &mut active_category,
                tabs_area,
                category_area,
            });
        assert!(handled);
        assert_eq!(active_tab, expected);
    }

    #[test]
    fn handle_tab_category_click_add_sets_pending_command() {
        let mut tabs = vec![TabState::new(
            "id1".into(),
            "默认".into(),
            "",
            false,
            "m1",
            "p1",
        )];
        let mut active_tab = 0usize;
        let categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let tabs_area = Rect::new(0, 0, 20, 1);
        let category_area = Rect::new(0, 2, 10, 5);

        let labels = crate::ui::runtime_helpers::tab_labels_for_category(&tabs, "默认");
        let active_pos = crate::ui::runtime_helpers::active_tab_position(&tabs, "默认", active_tab);
        let view = build_tab_bar_view(&labels, active_pos, tabs_area.width);
        let add_x = add_target_and_x(tabs_area, &view);

        let handled =
            handle_tab_category_click(crate::ui::runtime_events::TabCategoryClickParams {
                mouse_x: add_x,
                mouse_y: 0,
                tabs: &mut tabs,
                active_tab: &mut active_tab,
                categories: &categories,
                active_category: &mut active_category,
                tabs_area,
                category_area,
            });
        assert!(handled);
        assert_eq!(tabs[0].app.pending_command, Some(PendingCommand::NewTab));
    }

    #[test]
    fn handle_tab_category_click_ignores_outside() {
        let mut tabs = vec![TabState::new(
            "id1".into(),
            "默认".into(),
            "",
            false,
            "m1",
            "p1",
        )];
        let mut active_tab = 0usize;
        let categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let handled =
            handle_tab_category_click(crate::ui::runtime_events::TabCategoryClickParams {
                mouse_x: 50,
                mouse_y: 50,
                tabs: &mut tabs,
                active_tab: &mut active_tab,
                categories: &categories,
                active_category: &mut active_category,
                tabs_area: Rect::new(0, 0, 10, 1),
                category_area: Rect::new(0, 2, 10, 1),
            });
        assert!(!handled);
        assert_eq!(active_tab, 0);
        assert_eq!(active_category, 0);
    }

    fn more_right_target_and_x(area: Rect, view: &crate::ui::tab_bar::TabBarView) -> (usize, u16) {
        let mut cursor = area.x;
        for (i, item) in view.items.iter().enumerate() {
            if let TabBarItemKind::MoreRight { target_pos } = item.kind {
                return (target_pos, cursor);
            }
            cursor = cursor.saturating_add(item.label.width() as u16);
            if i + 1 < view.items.len() {
                cursor = cursor.saturating_add(1);
            }
        }
        panic!("expected MoreRight indicator in overflow view");
    }

    fn add_target_and_x(area: Rect, view: &crate::ui::tab_bar::TabBarView) -> u16 {
        let mut cursor = area.x;
        for (i, item) in view.items.iter().enumerate() {
            if let TabBarItemKind::Add = item.kind {
                return cursor;
            }
            cursor = cursor.saturating_add(item.label.width() as u16);
            if i + 1 < view.items.len() {
                cursor = cursor.saturating_add(1);
            }
        }
        panic!("expected Add item in tab bar view");
    }

    #[test]
    fn mouse_scroll_updates_scroll() {
        let mut ctx = base_mouse_ctx();
        ctx.tabs[0].app.scroll = 5;
        let m = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 1,
            row: 1,
            modifiers: KeyModifiers::NONE,
        };
        handle_mouse(&mut ctx, m);
        assert!(ctx.tabs[0].app.scroll < 5);
        let m = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 1,
            row: 1,
            modifiers: KeyModifiers::NONE,
        };
        handle_mouse(&mut ctx, m);
        assert!(ctx.tabs[0].app.scroll >= 5);
    }

    #[test]
    fn mouse_wheel_over_tabs_switches_tab_and_does_not_scroll_chat() {
        let mut ctx = base_mouse_ctx();
        ctx.tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id3".into(), "默认".into(), "", false, "m1", "p1"),
        ];
        ctx.categories = vec!["默认".to_string()];
        ctx.active_category = 0;
        ctx.active_tab = 0;
        ctx.tabs_area = Rect::new(0, 0, 40, 1);
        ctx.msg_area = Rect::new(0, 1, 40, 10);
        ctx.tabs[0].app.scroll = 7;

        let m = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 1,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        handle_mouse(&mut ctx, m);
        assert_eq!(ctx.active_tab, 1);
        assert_eq!(ctx.tabs[0].app.scroll, 7);

        let m = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 1,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        handle_mouse(&mut ctx, m);
        assert_eq!(ctx.active_tab, 0);
        assert_eq!(ctx.tabs[0].app.scroll, 7);
    }

    #[test]
    fn ctrl_c_copies_chat_selection() {
        let mut tabs = build_chat_tabs();
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let args = default_args();
        let handled =
            crate::ui::runtime_events::handle_key_event(key, &mut tabs, 0, &args, 40, &theme())
                .unwrap();
        assert!(!handled);
    }

    fn build_chat_tabs() -> Vec<TabState> {
        let mut tabs = vec![TabState::new(
            "id".into(),
            "默认".into(),
            "",
            false,
            "m1",
            "p1",
        )];
        tabs[0].app.focus = Focus::Chat;
        tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tabs[0].app.chat_selection = Some(crate::ui::selection::Selection {
            start: (0, 0),
            end: (0, 1),
        });
        tabs
    }

    fn default_args() -> crate::args::Args {
        crate::args::Args {
            model: None,
            system: "sys".to_string(),
            base_url: "http://example.com".to_string(),
            show_reasoning: false,
            resume: None,
            replay_fork_last: false,
            enable: None,
            log_requests: None,
            perf: false,
            question_set: None,
            workspace: "/tmp/deepchat-workspace".to_string(),
            yolo: false,
            read_only: false,
            wait_gdb: false,
        }
    }

    #[test]
    fn mouse_down_on_scrollbar_starts_dragging() {
        let mut ctx = base_mouse_ctx();
        let scroll_area = crate::ui::draw::scrollbar_area(ctx.msg_area);
        let m = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: scroll_area.x,
            row: scroll_area.y,
            modifiers: KeyModifiers::NONE,
        };
        handle_mouse(&mut ctx, m);
        assert!(ctx.tabs[0].app.scrollbar_dragging);
    }

    #[test]
    fn mouse_down_on_input_focuses_input() {
        let mut ctx = base_mouse_ctx();
        let m = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: ctx.input_area.x + 1,
            row: ctx.input_area.y + 1,
            modifiers: KeyModifiers::NONE,
        };
        handle_mouse(&mut ctx, m);
        assert_eq!(ctx.tabs[0].app.focus, Focus::Input);
        assert!(ctx.tabs[0].app.input_selecting);
    }
}
