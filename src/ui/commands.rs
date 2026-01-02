#[derive(Copy, Clone)]
pub(crate) struct CommandSpec {
    pub(crate) name: &'static str,
    pub(crate) args: &'static str,
    pub(crate) description: &'static str,
}

pub(crate) fn all_commands() -> &'static [CommandSpec] {
    &[
        CommandSpec {
            name: "/help",
            args: "",
            description: "打开帮助说明",
        },
        CommandSpec {
            name: "/save",
            args: "",
            description: "保存当前会话",
        },
        CommandSpec {
            name: "/reset",
            args: "",
            description: "清空对话（保留系统提示词）",
        },
        CommandSpec {
            name: "/clear",
            args: "",
            description: "清空对话（保留系统提示词）",
        },
        CommandSpec {
            name: "/exit",
            args: "",
            description: "退出应用",
        },
        CommandSpec {
            name: "/quit",
            args: "",
            description: "退出应用",
        },
        CommandSpec {
            name: "/category",
            args: "[name]",
            description: "新建分类并切换",
        },
        CommandSpec {
            name: "/open",
            args: "<id>",
            description: "打开指定对话",
        },
        CommandSpec {
            name: "/list-conv",
            args: "",
            description: "列出所有对话",
        },
    ]
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

pub(crate) fn command_names() -> Vec<&'static str> {
    all_commands().iter().map(|c| c.name).collect()
}

pub(crate) fn command_has_args(name: &str) -> bool {
    all_commands()
        .iter()
        .find(|c| c.name == name)
        .map(|c| !c.args.is_empty())
        .unwrap_or(false)
}
