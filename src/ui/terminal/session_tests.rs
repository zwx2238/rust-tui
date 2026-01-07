#[cfg(test)]
mod tests {
    #[test]
    fn vt100_parser_basic_output_renders_lines() {
        let mut p = vt100::Parser::new(5, 20, 100);
        p.process(b"hello\r\nworld\r\n");
        let s = p.screen().contents();
        assert!(s.contains("hello"));
        assert!(s.contains("world"));
    }
}

