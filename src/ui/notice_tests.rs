#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::notice::{draw_notice, push_notice};
    use crate::ui::state::{App, Notice};
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use ratatui::Terminal;
    use std::time::{Duration, Instant};

    fn theme() -> RenderTheme {
        RenderTheme {
            bg: Color::Black,
            fg: Some(Color::White),
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
    }

    #[test]
    fn push_notice_sets_text() {
        let mut app = App::new("", "m1", "p1");
        push_notice(&mut app, "hello");
        assert!(app.notice.as_ref().map(|n| n.text.as_str()) == Some("hello"));
    }

    #[test]
    fn draw_notice_clears_when_expired() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new("", "m1", "p1");
        app.notice = Some(Notice {
            text: "expired".to_string(),
            expires_at: Instant::now() - Duration::from_secs(1),
        });
        terminal
            .draw(|f| {
                draw_notice(f, Rect::new(0, 0, 40, 10), &mut app, &theme());
            })
            .unwrap();
        assert!(app.notice.is_none());
    }

    #[test]
    fn draw_notice_keeps_when_area_too_small() {
        let backend = TestBackend::new(10, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new("", "m1", "p1");
        app.notice = Some(Notice {
            text: "small".to_string(),
            expires_at: Instant::now() + Duration::from_secs(5),
        });
        terminal
            .draw(|f| {
                draw_notice(f, Rect::new(0, 0, 10, 2), &mut app, &theme());
            })
            .unwrap();
        assert!(app.notice.is_some());
    }

    #[test]
    fn draw_notice_renders_in_large_area() {
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new("", "m1", "p1");
        app.notice = Some(Notice {
            text: "hello notice".to_string(),
            expires_at: Instant::now() + Duration::from_secs(5),
        });
        terminal
            .draw(|f| {
                draw_notice(f, Rect::new(0, 0, 60, 10), &mut app, &theme());
            })
            .unwrap();
        assert!(app.notice.is_some());
    }
}
