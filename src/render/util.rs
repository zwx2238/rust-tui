use crate::types::{ROLE_ASSISTANT, ROLE_SYSTEM, ROLE_TOOL, ROLE_USER};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub(crate) fn ranges_overlap(start: usize, end: usize, a: usize, b: usize) -> bool {
    a < end && b > start
}

pub(crate) fn suffix_for_index(suffixes: &[(usize, String)], idx: usize) -> Option<&str> {
    suffixes
        .iter()
        .find(|(i, _)| *i == idx)
        .map(|(_, s)| s.as_str())
}

pub(crate) fn label_for_role(role: &str, suffix: Option<&str>) -> Option<String> {
    match role {
        ROLE_USER => Some("👤".to_string()),
        ROLE_ASSISTANT => {
            let mut label = "🤖".to_string();
            if let Some(s) = suffix
                && !s.is_empty()
            {
                label.push(' ');
                label.push_str(s);
            }
            Some(label)
        }
        ROLE_SYSTEM => Some("⚙️".to_string()),
        ROLE_TOOL => Some("🔧".to_string()),
        _ => None,
    }
}

pub(crate) fn hash_message(role: &str, content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    role.hash(&mut hasher);
    content.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::{hash_message, label_for_role, ranges_overlap, suffix_for_index};

    #[test]
    fn overlap_checks() {
        assert!(ranges_overlap(0, 5, 2, 3));
        assert!(!ranges_overlap(0, 5, 5, 6));
    }

    #[test]
    fn suffix_lookup() {
        let suffixes = vec![(1, "A".to_string()), (3, "B".to_string())];
        assert_eq!(suffix_for_index(&suffixes, 3), Some("B"));
        assert_eq!(suffix_for_index(&suffixes, 2), None);
    }

    #[test]
    fn label_for_role_handles_known() {
        assert!(label_for_role("user", None).is_some());
        assert!(label_for_role("unknown", None).is_none());
    }

    #[test]
    fn hash_message_changes() {
        let a = hash_message("user", "a");
        let b = hash_message("user", "b");
        assert_ne!(a, b);
    }
}
