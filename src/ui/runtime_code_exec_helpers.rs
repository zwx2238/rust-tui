pub(crate) fn inject_requirements(code: &str) -> String {
    let mut requirements = Vec::new();
    let mut code_lines = Vec::new();
    let mut in_header = true;
    for line in code.lines() {
        let trimmed = line.trim();
        if in_header && trimmed.is_empty() {
            code_lines.push(line);
            continue;
        }
        if in_header {
            if let Some(rest) = trimmed.strip_prefix("# requirements:") {
                let parts = rest.split(',').map(|s| s.trim()).filter(|s| !s.is_empty());
                for part in parts {
                    requirements.push(part.to_string());
                }
                continue;
            }
            in_header = false;
        }
        code_lines.push(line);
    }
    if requirements.is_empty() {
        return code.to_string();
    }
    let mut out = String::new();
    out.push_str("import subprocess, sys, os\n");
    out.push_str("work_dir = os.environ.get(\"DEEPCHAT_WORKDIR\", \"/opt/deepchat\")\n");
    out.push_str("tmp_dir = os.path.join(work_dir, \"tmp\")\n");
    out.push_str("site_dir = os.path.join(work_dir, \"site-packages\")\n");
    out.push_str("os.makedirs(tmp_dir, exist_ok=True)\n");
    out.push_str("os.makedirs(site_dir, exist_ok=True)\n");
    out.push_str("print(\"DEEPCHAT_PIP_BEGIN\")\n");
    out.push_str("subprocess.check_call([sys.executable, \"-m\", \"pip\", \"install\", \"--target\", site_dir");
    for req in &requirements {
        out.push_str(", \"");
        out.push_str(req);
        out.push('"');
    }
    out.push_str("])\n");
    out.push_str("print(\"DEEPCHAT_PIP_END\")\n");
    out.push_str("if site_dir not in sys.path:\n");
    out.push_str("    sys.path.insert(0, site_dir)\n");
    for line in code_lines {
        out.push_str(line);
        out.push('\n');
    }
    out
}

pub(crate) fn filter_pip_output(stdout: &str, exit_code: Option<i32>) -> String {
    if !pip_filter_allowed(exit_code) {
        return stdout.to_string();
    }
    let mut out = strip_pip_markers(stdout);
    trim_trailing_newline(&mut out);
    out
}

fn pip_filter_allowed(exit_code: Option<i32>) -> bool {
    matches!(exit_code, Some(0))
}

fn strip_pip_markers(stdout: &str) -> String {
    let mut out = String::new();
    let mut in_pip = false;
    for line in stdout.lines() {
        if line.trim() == "DEEPCHAT_PIP_BEGIN" {
            in_pip = true;
            continue;
        }
        if line.trim() == "DEEPCHAT_PIP_END" {
            in_pip = false;
            continue;
        }
        if !in_pip {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn trim_trailing_newline(out: &mut String) {
    if out.ends_with('\n') {
        out.pop();
        if out.ends_with('\r') {
            out.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{filter_pip_output, inject_requirements};

    #[test]
    fn inject_requirements_adds_pip_block() {
        let code = "# requirements: requests\nprint('hi')";
        let out = inject_requirements(code);
        assert!(out.contains("pip"));
        assert!(out.contains("requests"));
    }

    #[test]
    fn filter_pip_output_strips_on_success() {
        let stdout = "DEEPCHAT_PIP_BEGIN\npip stuff\nDEEPCHAT_PIP_END\nok\n";
        let out = filter_pip_output(stdout, Some(0));
        assert_eq!(out.trim(), "ok");
    }
}
