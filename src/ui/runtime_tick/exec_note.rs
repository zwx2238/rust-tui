use crate::ui::runtime_helpers::TabState;

pub fn build_exec_header_note(tabs: &[TabState], categories: &[String]) -> Option<String> {
    let mut pending_tabs = Vec::new();
    for (idx, tab) in tabs.iter().enumerate() {
        if !tab_has_pending_exec(tab) {
            continue;
        }
        let category = tab_category_name(tab, categories);
        let pos = crate::ui::runtime_helpers::tab_position_in_category(tabs, &category, idx)
            .unwrap_or(idx);
        pending_tabs.push(format!("{category}/对话{}", pos + 1));
    }
    if pending_tabs.is_empty() {
        return None;
    }
    let list = pending_tabs.join(", ");
    Some(format!("执行中: {} ({})", pending_tabs.len(), list))
}

fn tab_has_pending_exec(tab: &TabState) -> bool {
    tab.app.pending_code_exec.is_some() || tab.app.code_exec_live.is_some()
}

fn tab_category_name(tab: &TabState, categories: &[String]) -> String {
    if !tab.category.trim().is_empty() {
        return tab.category.clone();
    }
    categories
        .first()
        .cloned()
        .unwrap_or_else(|| "默认".to_string())
}
