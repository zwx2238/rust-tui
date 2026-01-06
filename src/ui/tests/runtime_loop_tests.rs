#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::render::RenderTheme;
    use crate::test_support::{env_lock, restore_env, set_env};
    use crate::ui::events::RuntimeEvent;
    use crate::ui::runtime_helpers::{PreheatTask, TabState};
    use crate::ui::runtime_loop::run_loop;
    use ratatui::Terminal;
    use ratatui::backend::CrosstermBackend;
    use ratatui::style::Color;
    use std::sync::mpsc;
    use std::time::Instant;

    struct RunLoopState {
        terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
        tabs: Vec<TabState>,
        active_tab: usize,
        categories: Vec<String>,
        active_category: usize,
        session_location: Option<crate::session::SessionLocation>,
        tx: mpsc::Sender<RuntimeEvent>,
        rx: mpsc::Receiver<RuntimeEvent>,
        preheat_tx: mpsc::Sender<PreheatTask>,
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

    fn base_run_loop_state() -> RunLoopState {
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout())).unwrap();
        let (tx, rx) = mpsc::channel::<RuntimeEvent>();
        let (preheat_tx, _preheat_rx) = mpsc::channel::<PreheatTask>();
        RunLoopState {
            terminal,
            tabs: vec![TabState::new(
                "id".into(),
                "默认".into(),
                "",
                false,
                "m1",
                "p1",
            )],
            active_tab: 0,
            categories: vec!["默认".to_string()],
            active_category: 0,
            session_location: None,
            tx,
            rx,
            preheat_tx,
        }
    }

    #[test]
    fn run_loop_exits_in_test_mode() {
        let _guard = env_lock().lock().unwrap();
        let prev = set_env("DEEPCHAT_TEST_RUN_LOOP_ONCE", "1");
        let mut state = base_run_loop_state();
        let result = run_loop(crate::ui::runtime_loop::RunLoopParams {
            terminal: &mut state.terminal,
            tabs: &mut state.tabs,
            active_tab: &mut state.active_tab,
            categories: &mut state.categories,
            active_category: &mut state.active_category,
            session_location: &mut state.session_location,
            rx: &state.rx,
            tx: &state.tx,
            preheat_tx: &state.preheat_tx,
            registry: &registry(),
            prompt_registry: &prompt_registry(),
            args: &args(),
            theme: &theme(),
            start_time: Instant::now(),
        });
        restore_env("DEEPCHAT_TEST_RUN_LOOP_ONCE", prev);
        assert!(result.is_ok());
    }
}
