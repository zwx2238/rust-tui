use fuzzy_matcher::FuzzyMatcher;

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

pub(crate) fn all_commands() -> &'static [CommandSpec] {
    &[
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

pub(crate) fn command_suggestions_for_input(
    line: &str,
    cursor_col: usize,
) -> Vec<CommandSuggestion> {
    if !line.starts_with('/') {
        return Vec::new();
    }
    let cursor = cursor_col.min(line.chars().count());
    let cmd_end_char = find_first_whitespace(line).unwrap_or(line.chars().count());
    if cursor <= cmd_end_char {
        let cursor_byte = byte_index_from_char(line, cursor);
        let pattern = line
            .get(1..cursor_byte)
            .unwrap_or("")
            .trim_start_matches('/');
        return build_command_name_suggestions(pattern);
    }
    let cmd_end_byte = byte_index_from_char(line, cmd_end_char);
    let cmd = line[..cmd_end_byte].trim_end();
    if !command_has_args(cmd) {
        return Vec::new();
    }
    let arg_start_char = first_non_whitespace(line, cmd_end_char).unwrap_or(cmd_end_char);
    let arg_start_byte = byte_index_from_char(line, arg_start_char);
    let cursor_byte = byte_index_from_char(line, cursor);
    let pattern = line.get(arg_start_byte..cursor_byte).unwrap_or("").trim();
    build_arg_suggestions(cmd, pattern)
}

fn build_command_name_suggestions(pattern: &str) -> Vec<CommandSuggestion> {
    if pattern.is_empty() {
        return all_commands()
            .iter()
            .map(|cmd| CommandSuggestion {
                label: command_usage(cmd),
                description: cmd.description.to_string(),
                insert: cmd.name.to_string(),
                kind: CommandSuggestionKind::Command,
            })
            .collect();
    }
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
    let pattern_lower = pattern.to_ascii_lowercase();
    let mut candidates = Vec::new();
    for (idx, cmd) in all_commands().iter().enumerate() {
        let name = cmd.name.trim_start_matches('/').to_ascii_lowercase();
        let rank = if name == pattern_lower {
            MatchRank::Exact
        } else if name.starts_with(&pattern_lower) {
            MatchRank::Prefix
        } else {
            MatchRank::Fuzzy
        };
        let score = match rank {
            MatchRank::Exact | MatchRank::Prefix => Some(0),
            MatchRank::Fuzzy => matcher.fuzzy_match(&name, &pattern_lower),
        };
        if let Some(score) = score {
            candidates.push((cmd, idx, rank, score));
        }
    }
    candidates.sort_by(|a, b| {
        a.2.cmp(&b.2)
            .then_with(|| b.3.cmp(&a.3))
            .then_with(|| a.1.cmp(&b.1))
    });
    candidates
        .into_iter()
        .map(|(cmd, _, _, _)| CommandSuggestion {
            label: command_usage(cmd),
            description: cmd.description.to_string(),
            insert: cmd.name.to_string(),
            kind: CommandSuggestionKind::Command,
        })
        .collect()
}

fn build_open_arg_suggestions(pattern: &str) -> Vec<CommandSuggestion> {
    let ids = list_conversation_ids().unwrap_or_default();
    if ids.is_empty() {
        return Vec::new();
    }
    let pattern_lower = pattern.to_ascii_lowercase();
    if pattern_lower.is_empty() {
        return ids
            .into_iter()
            .map(|id| CommandSuggestion {
                label: id.clone(),
                description: "对话 ID".to_string(),
                insert: id,
                kind: CommandSuggestionKind::Argument,
            })
            .collect();
    }
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
    let mut prefix = Vec::new();
    let mut fuzzy = Vec::new();
    for id in ids {
        let lower = id.to_ascii_lowercase();
        if lower.starts_with(&pattern_lower) {
            prefix.push(id);
        } else if let Some(score) = matcher.fuzzy_match(&lower, &pattern_lower) {
            fuzzy.push((id, score));
        }
    }
    prefix.sort();
    fuzzy.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    prefix
        .into_iter()
        .chain(fuzzy.into_iter().map(|(id, _)| id))
        .map(|id| CommandSuggestion {
            label: id.clone(),
            description: "对话 ID".to_string(),
            insert: id,
            kind: CommandSuggestionKind::Argument,
        })
        .collect()
}

fn command_usage(cmd: &CommandSpec) -> String {
    if cmd.args.is_empty() {
        cmd.name.to_string()
    } else {
        format!("{} {}", cmd.name, cmd.args)
    }
}

fn build_arg_suggestions(cmd: &str, pattern: &str) -> Vec<CommandSuggestion> {
    let Some(spec) = all_commands().iter().find(|c| c.name == cmd) else {
        return Vec::new();
    };
    let Some(provider) = spec.arg_provider else {
        return Vec::new();
    };
    match provider {
        ArgProvider::ConversationId => build_open_arg_suggestions(pattern),
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum MatchRank {
    Exact,
    Prefix,
    Fuzzy,
}

fn find_first_whitespace(line: &str) -> Option<usize> {
    line.chars().position(|ch| ch.is_whitespace())
}

fn first_non_whitespace(line: &str, start: usize) -> Option<usize> {
    line.chars()
        .enumerate()
        .skip(start)
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(idx, _)| idx)
}

fn byte_index_from_char(line: &str, char_idx: usize) -> usize {
    line.char_indices()
        .nth(char_idx)
        .map(|(idx, _)| idx)
        .unwrap_or(line.len())
}

#[cfg(test)]
mod tests {
    use super::command_suggestions_for_input;

    #[test]
    fn command_suggestions_rank_exact_prefix() {
        let items = command_suggestions_for_input("/he", 3);
        assert!(!items.is_empty());
        assert!(items[0].insert.starts_with("/help"));
    }
}
