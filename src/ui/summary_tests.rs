#[cfg(test)]
mod tests {
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::summary::{SummarySort, build_summary_rows, sort_summary_rows};

    #[test]
    fn build_summary_rows_includes_latest_user() {
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        tab.app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello world".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        let rows = build_summary_rows(&[tab], 10);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].latest_user, "hello w...");
    }

    #[test]
    fn sort_summary_rows_exec_time_prioritizes_pending() {
        let mut rows = vec![
            crate::ui::summary::SummaryRow {
                tab_index: 1,
                tab_id: 2,
                category: "c".to_string(),
                message_count: 0,
                status: "done",
                exec_pending: false,
                exec_since: None,
                latest_user: "-".to_string(),
            },
            crate::ui::summary::SummaryRow {
                tab_index: 0,
                tab_id: 1,
                category: "c".to_string(),
                message_count: 0,
                status: "done",
                exec_pending: true,
                exec_since: Some(std::time::Instant::now()),
                latest_user: "-".to_string(),
            },
        ];
        sort_summary_rows(&mut rows, SummarySort::ExecTime);
        assert!(rows[0].exec_pending);
    }

    #[test]
    fn sort_summary_rows_tab_order_by_index() {
        let mut rows = vec![
            crate::ui::summary::SummaryRow {
                tab_index: 2,
                tab_id: 3,
                category: "c".to_string(),
                message_count: 0,
                status: "done",
                exec_pending: false,
                exec_since: None,
                latest_user: "-".to_string(),
            },
            crate::ui::summary::SummaryRow {
                tab_index: 0,
                tab_id: 1,
                category: "c".to_string(),
                message_count: 0,
                status: "done",
                exec_pending: false,
                exec_since: None,
                latest_user: "-".to_string(),
            },
        ];
        sort_summary_rows(&mut rows, SummarySort::TabOrder);
        assert_eq!(rows[0].tab_index, 0);
    }
}
