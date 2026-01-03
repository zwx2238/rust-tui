use fuzzy_matcher::FuzzyMatcher;

use super::{
    ArgProvider, CommandSpec, CommandSuggestion, CommandSuggestionKind, all_commands,
    command_has_args, list_conversation_ids,
};

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
        let pattern = command_pattern_at_cursor(line, cursor);
        return build_command_name_suggestions(&pattern);
    }
    let cmd = command_name_from_line(line, cmd_end_char);
    if !command_has_args(cmd) {
        return Vec::new();
    }
    let pattern = arg_pattern_at_cursor(line, cursor, cmd_end_char);
    build_arg_suggestions(cmd, pattern)
}

fn command_pattern_at_cursor(line: &str, cursor: usize) -> String {
    let cursor_byte = byte_index_from_char(line, cursor);
    line.get(1..cursor_byte)
        .unwrap_or("")
        .trim_start_matches('/')
        .to_string()
}

fn command_name_from_line(line: &str, cmd_end_char: usize) -> &str {
    let cmd_end_byte = byte_index_from_char(line, cmd_end_char);
    line[..cmd_end_byte].trim_end()
}

fn arg_pattern_at_cursor(line: &str, cursor: usize, cmd_end_char: usize) -> &str {
    let arg_start_char = first_non_whitespace(line, cmd_end_char).unwrap_or(cmd_end_char);
    let arg_start_byte = byte_index_from_char(line, arg_start_char);
    let cursor_byte = byte_index_from_char(line, cursor);
    line.get(arg_start_byte..cursor_byte).unwrap_or("").trim()
}

fn build_command_name_suggestions(pattern: &str) -> Vec<CommandSuggestion> {
    if pattern.is_empty() {
        return all_commands()
            .iter()
            .map(command_to_suggestion)
            .collect();
    }
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
    let pattern_lower = pattern.to_ascii_lowercase();
    let mut candidates = collect_command_candidates(&pattern_lower, &matcher);
    candidates.sort_by(|a, b| {
        a.2.cmp(&b.2)
            .then_with(|| b.3.cmp(&a.3))
            .then_with(|| a.1.cmp(&b.1))
    });
    candidates
        .into_iter()
        .map(|(cmd, _, _, _)| command_to_suggestion(cmd))
        .collect()
}

fn command_to_suggestion(cmd: &CommandSpec) -> CommandSuggestion {
    CommandSuggestion {
        label: command_usage(cmd),
        description: cmd.description.to_string(),
        insert: cmd.name.to_string(),
        kind: CommandSuggestionKind::Command,
    }
}

fn collect_command_candidates<'a>(
    pattern_lower: &'a str,
    matcher: &'a fuzzy_matcher::skim::SkimMatcherV2,
) -> Vec<(&'a CommandSpec, usize, MatchRank, i64)> {
    let mut candidates = Vec::new();
    for (idx, cmd) in all_commands().iter().enumerate() {
        if let Some(candidate) = build_candidate(cmd, idx, pattern_lower, matcher) {
            candidates.push(candidate);
        }
    }
    candidates
}

fn build_candidate<'a>(
    cmd: &'a CommandSpec,
    idx: usize,
    pattern_lower: &'a str,
    matcher: &'a fuzzy_matcher::skim::SkimMatcherV2,
) -> Option<(&'a CommandSpec, usize, MatchRank, i64)> {
    let name = cmd.name.trim_start_matches('/').to_ascii_lowercase();
    let rank = match_rank(&name, pattern_lower);
    let score = match rank {
        MatchRank::Exact | MatchRank::Prefix => Some(0),
        MatchRank::Fuzzy => matcher.fuzzy_match(&name, pattern_lower),
    }?;
    Some((cmd, idx, rank, score))
}

fn match_rank(name: &str, pattern_lower: &str) -> MatchRank {
    if name == pattern_lower {
        MatchRank::Exact
    } else if name.starts_with(pattern_lower) {
        MatchRank::Prefix
    } else {
        MatchRank::Fuzzy
    }
}

fn build_open_arg_suggestions(pattern: &str) -> Vec<CommandSuggestion> {
    let ids = list_conversation_ids().unwrap_or_default();
    if ids.is_empty() {
        return Vec::new();
    }
    let pattern_lower = pattern.to_ascii_lowercase();
    if pattern_lower.is_empty() {
        return ids.into_iter().map(arg_to_suggestion).collect();
    }
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
    let (mut prefix, mut fuzzy) = split_arg_candidates(ids, &pattern_lower, &matcher);
    prefix.sort();
    fuzzy.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    prefix
        .into_iter()
        .chain(fuzzy.into_iter().map(|(id, _)| id))
        .map(arg_to_suggestion)
        .collect()
}

fn arg_to_suggestion(id: String) -> CommandSuggestion {
    CommandSuggestion {
        label: id.clone(),
        description: "对话 ID".to_string(),
        insert: id,
        kind: CommandSuggestionKind::Argument,
    }
}

fn split_arg_candidates(
    ids: Vec<String>,
    pattern_lower: &str,
    matcher: &fuzzy_matcher::skim::SkimMatcherV2,
) -> (Vec<String>, Vec<(String, i64)>) {
    let mut prefix = Vec::new();
    let mut fuzzy = Vec::new();
    for id in ids {
        let lower = id.to_ascii_lowercase();
        if lower.starts_with(pattern_lower) {
            prefix.push(id);
        } else if let Some(score) = matcher.fuzzy_match(&lower, pattern_lower) {
            fuzzy.push((id, score));
        }
    }
    (prefix, fuzzy)
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
