use serde::{Deserialize, Serialize};
use std::process::Command;
use std::thread;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct HookSpec {
    pub event: String,
    #[serde(alias = "command")]
    pub cmd: String,
}

pub const EVENT_LLM_DONE: &str = "llm.done";
pub const EVENT_LLM_ERROR: &str = "llm.error";
pub const EVENT_LLM_TOOL_CALLS: &str = "llm.tool_calls";
pub const EVENT_TOOL_BEFORE: &str = "tool.before";
pub const EVENT_TOOL_AFTER: &str = "tool.after";

pub fn run_hooks(hooks: &[HookSpec], event: &str, vars: Vec<(String, String)>) {
    for hook in hooks {
        if !hook_matches(hook, event) {
            continue;
        }
        if hook.cmd.trim().is_empty() {
            continue;
        }
        spawn_hook(hook.cmd.clone(), event.to_string(), vars.clone());
    }
}

fn hook_matches(hook: &HookSpec, event: &str) -> bool {
    hook.event.trim() == event
}

fn spawn_hook(cmd: String, event: String, vars: Vec<(String, String)>) {
    thread::spawn(move || {
        let mut process = Command::new("bash");
        process.arg("-lc").arg(cmd);
        process.env("HOOK_EVENT", event);
        for (key, value) in vars {
            process.env(key, value);
        }
        let _ = process.status();
    });
}
