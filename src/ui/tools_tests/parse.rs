use crate::ui::tools::parse_code_exec_args;

#[test]
fn parse_code_exec_rejects_empty() {
    assert!(parse_code_exec_args(r#"{"language":"","code":""}"#).is_err());
}

#[test]
fn parse_code_exec_rejects_invalid_json() {
    let err = parse_code_exec_args("{").err().unwrap();
    assert!(err.contains("参数解析失败"));
}

#[test]
fn parse_code_exec_rejects_non_python() {
    let err = parse_code_exec_args(r#"{"language":"js","code":"1"}"#)
        .err()
        .unwrap();
    assert!(err.contains("仅支持"));
}

#[test]
fn parse_code_exec_rejects_empty_code() {
    let err = parse_code_exec_args(r#"{"language":"python","code":"  "}"#)
        .err()
        .unwrap();
    assert!(err.contains("code 不能为空"));
}

#[test]
fn parse_code_exec_accepts_python() {
    let req = parse_code_exec_args(r#"{"language":"python","code":"print(1)"}"#).unwrap();
    assert_eq!(req.language, "python");
    assert!(req.code.contains("print"));
}
