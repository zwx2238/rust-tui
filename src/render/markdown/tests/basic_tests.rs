#[test]
fn closes_unbalanced_fence() {
    use super::super::close_unbalanced_code_fence;

    let input = "```\ncode";
    let out = close_unbalanced_code_fence(input);
    assert!(out.ends_with("```"));
}

#[test]
fn count_markdown_lines_basic() {
    use super::super::count_markdown_lines;

    let text = "# Title\n\n- a\n- b\n\n`code`";
    let count = count_markdown_lines(text, 40);
    assert!(count > 0);
}

#[test]
fn preprocess_math_keeps_plain() {
    use super::super::preprocess_math;

    let input = "no math here";
    let out = preprocess_math(input);
    assert_eq!(out, input);
}
