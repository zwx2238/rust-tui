#[cfg(test)]
mod tests {
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::render::RenderTheme;
    use crate::ui::render_context::RenderContext;
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_render::render_view;
    use crate::ui::runtime_view::ViewState;
    use ratatui::Terminal;
    use ratatui::backend::CrosstermBackend;
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use ratatui::text::{Line, Text};
    use std::io::Stdout;

    struct RenderTestState {
        terminal: Terminal<CrosstermBackend<Stdout>>,
        tabs: Vec<TabState>,
        view: ViewState,
        full_area: Rect,
        text: Text<'static>,
        tab_labels: Vec<String>,
        categories: Vec<String>,
        theme: RenderTheme,
        registry: ModelRegistry,
        prompt_registry: PromptRegistry,
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

    fn registry() -> ModelRegistry {
        ModelRegistry {
            default_key: "m1".to_string(),
            models: vec![ModelProfile {
                key: "m1".to_string(),
                base_url: "http://example.com".to_string(),
                api_key: "k".to_string(),
                model: "model".to_string(),
            }],
        }
    }

    fn prompt_registry() -> PromptRegistry {
        PromptRegistry {
            default_key: "p1".to_string(),
            prompts: vec![SystemPrompt {
                key: "p1".to_string(),
                content: "sys1".to_string(),
            }],
        }
    }

    fn make_terminal() -> Terminal<CrosstermBackend<Stdout>> {
        let backend = CrosstermBackend::new(std::io::stdout());
        Terminal::new(backend).unwrap()
    }

    fn full_area(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Rect {
        let size = terminal.size().unwrap_or(ratatui::layout::Size {
            width: 80,
            height: 24,
        });
        Rect::new(0, 0, size.width, size.height)
    }

    fn build_state() -> RenderTestState {
        let mut terminal = make_terminal();
        let full_area = full_area(&mut terminal);
        RenderTestState {
            terminal,
            tabs: vec![TabState::new(
                "id".into(),
                "默认".into(),
                "",
                false,
                "m1",
                "p1",
            )],
            view: ViewState::new(),
            full_area,
            text: Text::from(vec![Line::from("hello")]),
            tab_labels: vec![" 对话 1 ".to_string()],
            categories: vec!["默认".to_string()],
            theme: theme(),
            registry: registry(),
            prompt_registry: prompt_registry(),
        }
    }

    fn msg_area(full_area: Rect) -> Rect {
        Rect::new(0, 2, full_area.width, full_area.height.saturating_sub(5))
    }

    fn footer_area(full_area: Rect) -> Rect {
        Rect::new(0, full_area.height.saturating_sub(1), full_area.width, 1)
    }

    fn render_context_and_view<'a>(
        state: &'a mut RenderTestState,
    ) -> (RenderContext<'a>, &'a mut ViewState) {
        let RenderTestState {
            terminal,
            tabs,
            view,
            full_area,
            text,
            tab_labels,
            categories,
            theme,
            registry,
            prompt_registry,
        } = state;
        let areas = render_areas(*full_area);
        let ctx = RenderContext {
            terminal, tabs,
            active_tab: 0, tab_labels, active_tab_pos: 0,
            categories, active_category: 0,
            theme, startup_text: None,
            full_area: *full_area, input_height: 3,
            msg_area: areas.msg_area, tabs_area: areas.tabs_area, category_area: areas.category_area,
            header_area: areas.header_area, footer_area: areas.footer_area, msg_width: 40,
            text, total_lines: 1, header_note: None,
            models: &registry.models, prompts: &prompt_registry.prompts,
        };
        (ctx, view)
    }

    struct RenderAreas {
        msg_area: Rect,
        tabs_area: Rect,
        category_area: Rect,
        header_area: Rect,
        footer_area: Rect,
    }

    fn render_areas(full_area: Rect) -> RenderAreas {
        RenderAreas {
            msg_area: msg_area(full_area),
            tabs_area: Rect::new(0, 1, full_area.width, 1),
            category_area: Rect::new(0, 1, 10, 5),
            header_area: Rect::new(0, 0, full_area.width, 1),
            footer_area: footer_area(full_area),
        }
    }

    #[test]
    fn render_view_chat_smoke() {
        let mut state = build_state();
        let (mut ctx, view) = render_context_and_view(&mut state);
        let _ = render_view(&mut ctx, view).unwrap();
    }

    #[test]
    fn render_view_summary_smoke() {
        let mut state = build_state();
        state
            .view
            .overlay
            .open(crate::ui::overlay::OverlayKind::Summary);
        let (mut ctx, view) = render_context_and_view(&mut state);
        let _ = render_view(&mut ctx, view).unwrap();
    }

    #[test]
    fn render_view_tool_overlays_smoke() {
        let mut state = build_state();
        state.tabs[0].app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
            call_id: "call1".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: std::time::Instant::now(),
            stop_reason: None,
        });
        state.tabs[0].app.pending_file_patch = Some(crate::ui::state::PendingFilePatch {
            call_id: "call2".to_string(),
            path: None,
            diff: "diff --git a/a b/a".to_string(),
            preview: "preview".to_string(),
        });
        state
            .view
            .overlay
            .open(crate::ui::overlay::OverlayKind::CodeExec);
        let (mut ctx, view) = render_context_and_view(&mut state);
        let _ = render_view(&mut ctx, view).unwrap();
        state
            .view
            .overlay
            .open(crate::ui::overlay::OverlayKind::FilePatch);
        let (mut ctx, view) = render_context_and_view(&mut state);
        let _ = render_view(&mut ctx, view).unwrap();
    }

    #[test]
    fn render_view_other_overlays_smoke() {
        let mut state = build_state();
        state.tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        for kind in [
            crate::ui::overlay::OverlayKind::Jump,
            crate::ui::overlay::OverlayKind::Model,
            crate::ui::overlay::OverlayKind::Prompt,
            crate::ui::overlay::OverlayKind::Help,
        ] {
            state.view.overlay.open(kind);
            let (mut ctx, view) = render_context_and_view(&mut state);
            let _ = render_view(&mut ctx, view).unwrap();
        }
    }
}
