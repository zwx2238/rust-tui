#[path = "commands_suggestions.rs"]
mod commands_suggestions;

pub(crate) use commands_suggestions::command_suggestions_for_input;

#[derive(Copy, Clone)]
pub(crate) struct CommandSpec {
    pub(crate) name: &'static str,
    pub(crate) args: &'static str,
    pub(crate) description: &'static str,
    pub(crate) arg_provider: Option<ArgProvider>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CommandSuggestion {
    pub(crate) label: String,
    pub(crate) description: String,
    pub(crate) insert: String,
    pub(crate) kind: CommandSuggestionKind,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum CommandSuggestionKind {
    Command,
    Argument,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum ArgProvider {
    ConversationId,
}

const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "/help",
        args: "",
        description: "打开帮助说明",
        arg_provider: None,
    },
    CommandSpec {
        name: "/save",
        args: "",
        description: "保存当前会话",
        arg_provider: None,
    },
    CommandSpec {
        name: "/reset",
        args: "",
        description: "清空对话（保留系统提示词）",
        arg_provider: None,
    },
    CommandSpec {
        name: "/clear",
        args: "",
        description: "清空对话（保留系统提示词）",
        arg_provider: None,
    },
    CommandSpec {
        name: "/exit",
        args: "",
        description: "退出应用",
        arg_provider: None,
    },
    CommandSpec {
        name: "/quit",
        args: "",
        description: "退出应用",
        arg_provider: None,
    },
    CommandSpec {
        name: "/category",
        args: "[name]",
        description: "新建分类并切换",
        arg_provider: None,
    },
    CommandSpec {
        name: "/open",
        args: "<id>",
        description: "打开指定对话",
        arg_provider: Some(ArgProvider::ConversationId),
    },
    CommandSpec {
        name: "/list-conv",
        args: "",
        description: "列出所有对话",
        arg_provider: None,
    },
];

pub(crate) fn all_commands() -> &'static [CommandSpec] {
    COMMANDS
}

pub(crate) fn commands_help_text() -> String {
    let mut lines = Vec::new();
    for cmd in all_commands() {
        let usage = if cmd.args.is_empty() {
            cmd.name.to_string()
        } else {
            format!("{} {}", cmd.name, cmd.args)
        };
        lines.push(format!("{usage}  -  {}", cmd.description));
    }
    format!("可用命令：\n{}", lines.join("\n"))
}

pub(crate) fn command_has_args(name: &str) -> bool {
    all_commands()
        .iter()
        .find(|c| c.name == name)
        .map(|c| !c.args.is_empty())
        .unwrap_or(false)
}

pub(crate) fn list_conversation_ids() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let dir = crate::conversation::conversations_dir()?;
    let entries = std::fs::read_dir(dir)?;
    let mut ids = Vec::new();
    for entry in entries.flatten() {
        if let Some(stem) = entry.path().file_stem() {
            ids.push(stem.to_string_lossy().to_string());
        }
    }
    ids.sort();
    Ok(ids)
}
