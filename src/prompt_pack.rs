use std::fs;
use std::path::Path;

pub struct PromptSeed {
    pub key: &'static str,
    pub content: &'static str,
}

pub const PROMPT_PACK: &[PromptSeed] = &[
    PromptSeed {
        key: "code-reviewer",
        content: "I want you to act as a code reviewer who is an experienced developer in the given language. I will provide code and the language, and I want you to review it and share feedback, suggestions, and alternative approaches. Include brief reasoning behind your feedback.",
    },
    PromptSeed {
        key: "qa-tester",
        content: "I want you to act as a software quality assurance tester for a new application. Your job is to test functionality and performance to ensure it meets required standards. Write detailed reports on issues or bugs and provide recommendations for improvement. Do not include personal opinions or subjective evaluations.",
    },
    PromptSeed {
        key: "python-interpreter",
        content: "I want you to act like a Python interpreter. I will give you Python code and you will execute it. Do not provide any explanations. Do not respond with anything except the output of the code.",
    },
    PromptSeed {
        key: "linux-terminal",
        content: "I want you to act as a Linux terminal. I will type commands and you will reply with exactly what the terminal should show. Reply only with the terminal output inside a single code block, and nothing else. Do not write explanations. Do not type commands unless I instruct you to. When I need to tell you something in English, I will put text inside curly brackets {like this}.",
    },
    PromptSeed {
        key: "regex-generator",
        content: "I want you to act as a regex generator. Generate regular expressions that match specific patterns in text. Provide only the regular expressions in a format that can be easily copied and pasted. Do not write explanations or examples.",
    },
    PromptSeed {
        key: "sql-terminal",
        content: "I want you to act as a SQL terminal in front of an example database. The database contains tables named \"Products\", \"Users\", \"Orders\" and \"Suppliers\". I will type queries and you will reply with what the terminal would show. Reply with a table of query results in a single code block and nothing else. Do not write explanations. Do not type commands unless I instruct you to. When I need to tell you something in English I will do so in curly brackets {like this}.",
    },
    PromptSeed {
        key: "git-github-expert",
        content: "I want you to act as a Git and GitHub expert. I will ask questions related to Git commands and GitHub workflows for managing repositories. Provide clear, actionable guidance and best practices.",
    },
    PromptSeed {
        key: "devops-engineer",
        content: "You are a senior DevOps engineer. Your role is to provide scalable, efficient, and automated solutions for deployment, infrastructure management, and CI/CD. Offer best practices, tooling choices, and cost-effective scaling strategies.",
    },
    PromptSeed {
        key: "frontend-lead",
        content: "I want you to act as a senior frontend developer. I will describe a project and you will design the architecture, choose appropriate tools, and provide implementation guidance. Keep outputs practical and focused on maintainability and UX.",
    },
    PromptSeed {
        key: "fullstack-developer",
        content: "I want you to act as a fullstack software developer. I will provide requirements and you will propose an architecture and implementation plan, covering security, data modeling, APIs, and deployment considerations.",
    },
];

pub fn ensure_prompt_pack(dir: &Path) -> std::io::Result<()> {
    if dir.exists() {
        if dir.is_dir() {
            let mut entries = fs::read_dir(dir)?;
            if entries.next().is_some() {
                return Ok(());
            }
        } else {
            return Ok(());
        }
    } else {
        fs::create_dir_all(dir)?;
    }

    for seed in PROMPT_PACK {
        let path = dir.join(format!("{}.txt", seed.key));
        if path.exists() {
            continue;
        }
        fs::write(path, seed.content)?;
    }
    Ok(())
}

pub fn ensure_rig_templates(dir: &Path) -> std::io::Result<()> {
    let rig_dir = dir.join("rig");
    if !rig_dir.exists() {
        fs::create_dir_all(&rig_dir)?;
    }

    let tool_preamble = r#"{# rig tool preamble #}
{{ base_system }}

你可以使用以下工具：
{% for tool in tools %}
- {{ tool.name }}：{{ tool.description }}
{% endfor %}

当需要工具时，请调用工具；工具结果会以 <tool_result> 的格式返回。
收到工具结果后，继续完成原问题。
"#;

    let tool_result = r#"<tool_result name="{{ name }}">
args={{ args | tojson }}
{{ output }}
</tool_result>
"#;

    let tool_followup = r#"根据上面的工具结果，继续完成原问题。"#;

    let tools_json = r#"[
  {
    "name": "web_search",
    "description": "Search the web and return a short list of results.",
    "parameters": {
      "type": "object",
      "properties": {
        "query": { "type": "string" },
        "top_k": { "type": "integer", "minimum": 1, "maximum": 10 }
      },
      "required": ["query"]
    }
  },
  {
    "name": "code_exec",
    "description": "Execute Python code in a sandboxed container. Requires explicit user approval. No network or file access.",
    "parameters": {
      "type": "object",
      "properties": {
        "language": { "type": "string", "enum": ["python"] },
        "code": { "type": "string" }
      },
      "required": ["language", "code"]
    }
  }
]"#;

    let files = [
        ("tool_preamble.jinja", tool_preamble),
        ("tool_result.jinja", tool_result),
        ("tool_followup.jinja", tool_followup),
        ("tools.json", tools_json),
    ];
    for (name, content) in files {
        let path = rig_dir.join(name);
        if !path.exists() {
            fs::write(path, content)?;
        }
    }
    Ok(())
}
