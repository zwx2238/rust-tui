#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::render::RenderTheme;
    use crate::session::SessionLocation;
    use crate::ui::net::UiEvent;
    use crate::ui::overlay::OverlayKind;
    use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState};
    use crate::ui::runtime_view::ViewState;
    use crate::ui::widget_system::{EventCtx, LayoutCtx, UpdateCtx, WidgetSystem};
    use ratatui::Terminal;
    use ratatui::backend::CrosstermBackend;
    use ratatui::style::Color;
    use std::io::Stdout;
    use std::sync::mpsc;

    struct WidgetTestState {
        terminal: Terminal<CrosstermBackend<Stdout>>,
        tabs: Vec<TabState>,
        active_tab: usize,
        categories: Vec<String>,
        active_category: usize,
        view: ViewState,
        registry: ModelRegistry,
        prompt_registry: PromptRegistry,
        args: Args,
        theme: RenderTheme,
        rx: mpsc::Receiver<UiEvent>,
        tx: mpsc::Sender<UiEvent>,
        preheat_tx: mpsc::Sender<PreheatTask>,
        preheat_res_rx: mpsc::Receiver<PreheatResult>,
        session_location: Option<SessionLocation>,
        startup_elapsed: Option<std::time::Duration>,
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

    fn args() -> Args {
        Args {
            model: "m".to_string(),
            system: "sys".to_string(),
            base_url: "http://example.com".to_string(),
            show_reasoning: false,
            config: None,
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

    fn build_state() -> WidgetTestState {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend).unwrap();
        let (tx, rx) = mpsc::channel();
        let (preheat_tx, _preheat_rx) = mpsc::channel();
        let (_preheat_res_tx, preheat_res_rx) = mpsc::channel();
        WidgetTestState {
            terminal,
            tabs: vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")],
            active_tab: 0,
            categories: vec!["默认".to_string()],
            active_category: 0,
            view: ViewState::new(),
            registry: registry(),
            prompt_registry: prompt_registry(),
            args: args(),
            theme: theme(),
            rx,
            tx,
            preheat_tx,
            preheat_res_rx,
            session_location: None,
            startup_elapsed: None,
        }
    }

    fn layout_ctx<'a>(state: &'a mut WidgetTestState) -> LayoutCtx<'a> {
        LayoutCtx {
            terminal: &mut state.terminal,
            view: &state.view,
            tabs: &state.tabs,
            active_tab: state.active_tab,
            categories: &state.categories,
        }
    }

    fn update_ctx<'a>(state: &'a mut WidgetTestState) -> UpdateCtx<'a> {
        UpdateCtx {
            tabs: &mut state.tabs,
            active_tab: &mut state.active_tab,
            categories: &mut state.categories,
            active_category: &mut state.active_category,
            session_location: &mut state.session_location,
            rx: &state.rx,
            tx: &state.tx,
            preheat_tx: &state.preheat_tx,
            preheat_res_rx: &state.preheat_res_rx,
            registry: &state.registry,
            prompt_registry: &state.prompt_registry,
            args: &state.args,
            theme: &state.theme,
            startup_elapsed: &mut state.startup_elapsed,
            view: &mut state.view,
        }
    }

    fn event_ctx<'a>(state: &'a mut WidgetTestState) -> EventCtx<'a> {
        EventCtx {
            tabs: &mut state.tabs,
            active_tab: &mut state.active_tab,
            categories: &mut state.categories,
            active_category: &mut state.active_category,
            theme: &state.theme,
            registry: &state.registry,
            prompt_registry: &state.prompt_registry,
            args: &state.args,
            view: &mut state.view,
        }
    }

    fn run_cycle(state: &mut WidgetTestState, system: &mut WidgetSystem) {
        let layout = {
            let mut ctx = layout_ctx(state);
            system.layout(&mut ctx).unwrap()
        };
        let update = {
            let mut ctx = update_ctx(state);
            system.update(&mut ctx, &layout).unwrap()
        };
        let mut ctx = event_ctx(state);
        let _ = system.event(&mut ctx, &layout, &update, &[]).unwrap();
    }

    #[test]
    fn widget_system_lifecycle_overlays_smoke() {
        let mut state = build_state();
        let mut system = WidgetSystem::new();
        let overlays = [
            None,
            Some(OverlayKind::Summary),
            Some(OverlayKind::Jump),
            Some(OverlayKind::Model),
            Some(OverlayKind::Prompt),
            Some(OverlayKind::CodeExec),
            Some(OverlayKind::FilePatch),
            Some(OverlayKind::Help),
        ];
        for overlay in overlays {
            match overlay {
                Some(kind) => state.view.overlay.open(kind),
                None => state.view.overlay.close(),
            }
            run_cycle(&mut state, &mut system);
        }
    }
}
