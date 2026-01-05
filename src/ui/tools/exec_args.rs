use super::CodeExecRequest;

pub(crate) fn parse_code_exec_args(args_json: &str) -> Result<CodeExecRequest, String> {
    #[derive(serde::Deserialize)]
    struct Args {
        language: String,
        code: String,
    }
    let args: Args =
        serde_json::from_str(args_json).map_err(|e| format!("code_exec 参数解析失败：{e}"))?;
    let language = args.language.trim().to_string();
    if language.is_empty() {
        return Err("code_exec 参数 language 不能为空".to_string());
    }
    if language != "python" {
        return Err("当前仅支持 python".to_string());
    }
    if args.code.trim().is_empty() {
        return Err("code_exec 参数 code 不能为空".to_string());
    }
    Ok(CodeExecRequest {
        language,
        code: args.code,
    })
}

pub(crate) fn parse_bash_exec_args(args_json: &str) -> Result<CodeExecRequest, String> {
    #[derive(serde::Deserialize)]
    struct Args {
        command: Option<String>,
        code: Option<String>,
    }
    let args: Args =
        serde_json::from_str(args_json).map_err(|e| format!("bash_exec 参数解析失败：{e}"))?;
    let command = args
        .command
        .or(args.code)
        .unwrap_or_default()
        .trim()
        .to_string();
    if command.is_empty() {
        return Err("bash_exec 参数 command 不能为空".to_string());
    }
    Ok(CodeExecRequest {
        language: "bash".to_string(),
        code: command,
    })
}
