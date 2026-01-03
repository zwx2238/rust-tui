#[cfg(test)]
mod tests {
    use crate::ui::shortcuts::all_shortcuts;

    #[test]
    fn shortcuts_list_has_entries() {
        let items = all_shortcuts();
        assert!(!items.is_empty());
        assert!(items.iter().any(|s| s.keys.contains("Ctrl+Q")));
    }
}
