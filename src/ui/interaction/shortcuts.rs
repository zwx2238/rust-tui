#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum ShortcutScope {
    Global,
    Chat,
    Nav,
    Summary,
    Jump,
    Model,
    Prompt,
    QuestionReview,
    CodeExec,
    Help,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Shortcut {
    pub(crate) scope: ShortcutScope,
    pub(crate) keys: &'static str,
    pub(crate) description: &'static str,
}

const SHORTCUTS: &[Shortcut] = &[
    Shortcut {
        scope: ShortcutScope::Global,
        keys: "Ctrl+Q",
        description: "退出应用",
    },
    Shortcut {
        scope: ShortcutScope::Global,
        keys: "Ctrl+T",
        description: "新建对话",
    },
    Shortcut {
        scope: ShortcutScope::Global,
        keys: "Ctrl+W",
        description: "关闭当前对话",
    },
    Shortcut {
        scope: ShortcutScope::Global,
        keys: "Ctrl+Shift+W",
        description: "关闭所有对话（保留空对话）",
    },
    Shortcut {
        scope: ShortcutScope::Global,
        keys: "Ctrl+O",
        description: "关闭其他对话",
    },
    Shortcut {
        scope: ShortcutScope::Global,
        keys: "F8 / F9",
        description: "前一个 / 下一个对话（Tab 过多可用 Tab 栏 «/» 或 F1 汇总页）",
    },
    Shortcut {
        scope: ShortcutScope::Global,
        keys: "Ctrl+↑ / Ctrl+↓",
        description: "切换分类",
    },
    Shortcut {
        scope: ShortcutScope::Chat,
        keys: "F1",
        description: "汇总页",
    },
    Shortcut {
        scope: ShortcutScope::Chat,
        keys: "F2",
        description: "消息跳转（右侧历史可点击）",
    },
    Shortcut {
        scope: ShortcutScope::Chat,
        keys: "F3",
        description: "切换模型",
    },
    Shortcut {
        scope: ShortcutScope::Chat,
        keys: "F4",
        description: "模型列表",
    },
    Shortcut {
        scope: ShortcutScope::Chat,
        keys: "F5",
        description: "系统提示词列表",
    },
    Shortcut {
        scope: ShortcutScope::Chat,
        keys: "F6",
        description: "终止生成",
    },
    Shortcut {
        scope: ShortcutScope::Chat,
        keys: "Shift+F6",
        description: "终止生成并编辑",
    },
    Shortcut {
        scope: ShortcutScope::Chat,
        keys: "F7",
        description: "弹窗终端",
    },
    Shortcut {
        scope: ShortcutScope::Chat,
        keys: "F10",
        description: "快捷键帮助",
    },
    Shortcut {
        scope: ShortcutScope::Nav,
        keys: "g",
        description: "进入/退出导航模式",
    },
    Shortcut {
        scope: ShortcutScope::Nav,
        keys: "j / n",
        description: "跳到下一条消息",
    },
    Shortcut {
        scope: ShortcutScope::Nav,
        keys: "k / p",
        description: "跳到上一条消息",
    },
    Shortcut {
        scope: ShortcutScope::Summary,
        keys: "↑/↓",
        description: "选择对话",
    },
    Shortcut {
        scope: ShortcutScope::Summary,
        keys: "Enter",
        description: "进入对话",
    },
    Shortcut {
        scope: ShortcutScope::Summary,
        keys: "Esc",
        description: "关闭汇总页",
    },
    Shortcut {
        scope: ShortcutScope::Jump,
        keys: "↑/↓",
        description: "选择消息",
    },
    Shortcut {
        scope: ShortcutScope::Jump,
        keys: "PageUp/PageDown",
        description: "翻页",
    },
    Shortcut {
        scope: ShortcutScope::Jump,
        keys: "Enter",
        description: "跳转到消息",
    },
    Shortcut {
        scope: ShortcutScope::Jump,
        keys: "E",
        description: "分叉并编辑消息",
    },
    Shortcut {
        scope: ShortcutScope::Model,
        keys: "↑/↓",
        description: "选择模型",
    },
    Shortcut {
        scope: ShortcutScope::Model,
        keys: "Enter",
        description: "确认模型",
    },
    Shortcut {
        scope: ShortcutScope::Prompt,
        keys: "↑/↓",
        description: "选择系统提示词",
    },
    Shortcut {
        scope: ShortcutScope::Prompt,
        keys: "Enter",
        description: "确认系统提示词",
    },
    Shortcut {
        scope: ShortcutScope::QuestionReview,
        keys: "↑/↓",
        description: "选择问题",
    },
    Shortcut {
        scope: ShortcutScope::QuestionReview,
        keys: "Space / Y / N",
        description: "通过 / 拒绝当前问题",
    },
    Shortcut {
        scope: ShortcutScope::QuestionReview,
        keys: "A / R",
        description: "全部通过 / 全部拒绝",
    },
    Shortcut {
        scope: ShortcutScope::QuestionReview,
        keys: "m / M",
        description: "切换模型 / 全部使用当前模型",
    },
    Shortcut {
        scope: ShortcutScope::QuestionReview,
        keys: "PgUp / PgDn",
        description: "滚动详情",
    },
    Shortcut {
        scope: ShortcutScope::QuestionReview,
        keys: "Enter / Esc",
        description: "提交 / 取消",
    },
    Shortcut {
        scope: ShortcutScope::CodeExec,
        keys: "鼠标点击",
        description: "确认/拒绝/停止/退出",
    },
    Shortcut {
        scope: ShortcutScope::Global,
        keys: "鼠标点击 Tab 栏 «/»",
        description: "Tab 过多时左右跳转到被隐藏的对话",
    },
    Shortcut {
        scope: ShortcutScope::Help,
        keys: "Esc / F10",
        description: "关闭帮助",
    },
];

pub(crate) fn all_shortcuts() -> &'static [Shortcut] {
    SHORTCUTS
}
