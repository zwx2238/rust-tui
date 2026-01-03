use crate::types::{ROLE_ASSISTANT, ROLE_USER};
use crate::ui::state::App;

const PERF_MESSAGES: usize = 50;
const PERF_LINES_PER_MESSAGE: usize = 100;

pub fn seed_perf_messages(app: &mut App) {
    for i in 0..PERF_MESSAGES {
        let role = role_for_index(i);
        let content = build_perf_content(i);
        app.messages.push(crate::types::Message {
            role: role.to_string(),
            content,
            tool_call_id: None,
            tool_calls: None,
        });
    }
    app.follow = true;
    app.scroll = u16::MAX;
}

fn role_for_index(i: usize) -> &'static str {
    if i.is_multiple_of(2) {
        ROLE_USER
    } else {
        ROLE_ASSISTANT
    }
}

fn build_perf_content(i: usize) -> String {
    let mut content = String::new();
    for line in 0..PERF_LINES_PER_MESSAGE {
        if line % 7 == 0 {
            content.push_str(&format!(
                "```rust\nfn demo_{i}_{line}() {{\n    // 注释 {i}-{line}\n    let x = {line};\n    println!(\"{}\", x);\n}}\n```\n",
                "{x}"
            ));
        } else if line % 7 == 1 {
            content.push_str(&format!(
                "这是一段较长的正文 {i}-{line}。包含多行文字，模拟真实对话内容。\n"
            ));
        } else {
            content.push_str(&format!("普通行 {i}-{line}\n"));
        }
    }
    content
}

#[cfg(test)]
mod tests {
    use super::seed_perf_messages;
    use crate::ui::state::App;

    #[test]
    fn seed_perf_messages_populates_app() {
        let mut app = App::new("", "m1", "p1");
        seed_perf_messages(&mut app);
        assert_eq!(app.messages.len(), 50);
        assert!(app.follow);
        assert_eq!(app.scroll, u16::MAX);
    }
}
