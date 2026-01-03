use minijinja::{Environment, Value, context};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

pub struct RigTemplates {
    env: Environment<'static>,
    root: PathBuf,
}

impl RigTemplates {
    pub fn load(prompts_dir: &str) -> Result<Self, String> {
        let root = Path::new(prompts_dir).join("rig");
        if !root.exists() {
            return Err(format!("缺少工具模板目录：{}", root.display()));
        }
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader(root.clone()));
        env.add_filter(
            "tojson",
            |value: Value| -> Result<String, minijinja::Error> { Ok(value.to_string()) },
        );
        Ok(Self { env, root })
    }

    pub fn tool_defs(&self) -> Result<Vec<ToolSchema>, String> {
        let path = self.root.join("tools.json");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取工具定义失败：{} ({e})", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("解析工具定义失败：{e}"))
    }

    pub fn render_preamble(
        &self,
        base_system: &str,
        tools: &[ToolSchema],
    ) -> Result<String, String> {
        let tmpl = self
            .env
            .get_template("tool_preamble.jinja")
            .map_err(|e| format!("加载模板失败：{e}"))?;
        tmpl.render(context! { base_system => base_system, tools => tools })
            .map_err(|e| format!("渲染模板失败：{e}"))
    }

    pub fn render_tool_result(
        &self,
        name: &str,
        args: &serde_json::Value,
        output: &str,
    ) -> Result<String, String> {
        let tmpl = self
            .env
            .get_template("tool_result.jinja")
            .map_err(|e| format!("加载模板失败：{e}"))?;
        tmpl.render(context! { name => name, args => args, output => output })
            .map_err(|e| format!("渲染模板失败：{e}"))
    }

    pub fn render_followup(&self) -> Result<String, String> {
        let tmpl = self
            .env
            .get_template("tool_followup.jinja")
            .map_err(|e| format!("加载模板失败：{e}"))?;
        tmpl.render(context! {})
            .map_err(|e| format!("渲染模板失败：{e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let dir = std::env::temp_dir().join(format!("deepchat_{name}_{ts}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_rig_templates(root: &Path) {
        let rig = root.join("rig");
        fs::create_dir_all(&rig).unwrap();
        fs::write(
            rig.join("tools.json"),
            r#"[{"name":"code_exec","description":"exec","parameters":{"type":"object"}}]"#,
        )
        .unwrap();
        fs::write(
            rig.join("tool_preamble.jinja"),
            "BASE={{ base_system }} TOOLS={{ tools|length }}",
        )
        .unwrap();
        fs::write(
            rig.join("tool_result.jinja"),
            "RESULT={{ name }} {{ args|tojson }} {{ output }}",
        )
        .unwrap();
        fs::write(rig.join("tool_followup.jinja"), "FOLLOWUP").unwrap();
    }

    #[test]
    fn load_requires_rig_dir() {
        let dir = temp_dir("rig_missing");
        let err = RigTemplates::load(dir.to_string_lossy().as_ref());
        let msg = match err {
            Ok(_) => String::new(),
            Err(e) => e,
        };
        assert!(msg.contains("缺少工具模板目录"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn renders_templates_and_loads_tools() {
        let dir = temp_dir("rig_ok");
        write_rig_templates(&dir);
        let templates = RigTemplates::load(dir.to_string_lossy().as_ref()).unwrap();
        let tools = templates.tool_defs().unwrap();
        assert_eq!(tools.len(), 1);
        let preamble = templates.render_preamble("SYS", &tools).unwrap();
        assert!(preamble.contains("BASE=SYS"));
        let result = templates
            .render_tool_result("code_exec", &serde_json::json!({"x": 1}), "ok")
            .unwrap();
        assert!(result.contains("code_exec"));
        assert!(result.contains("ok"));
        let follow = templates.render_followup().unwrap();
        assert_eq!(follow, "FOLLOWUP");
        let _ = fs::remove_dir_all(&dir);
    }
}
