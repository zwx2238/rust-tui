use crate::args::Args;
use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::runtime::state::PendingCommand;

pub(crate) struct HandleTabCommandParams<'a> {
    pub(crate) pending: PendingCommand,
    pub(crate) tabs: &'a mut Vec<TabState>,
    pub(crate) active_tab: &'a mut usize,
    pub(crate) categories: &'a mut Vec<String>,
    pub(crate) active_category: &'a mut usize,
    pub(crate) registry: &'a crate::model_registry::ModelRegistry,
    pub(crate) prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub(crate) args: &'a Args,
}

pub(crate) fn handle_tab_command(params: HandleTabCommandParams<'_>) {
    match params.pending {
        PendingCommand::NewTab => {
            crate::framework::widget_system::runtime_loop_helpers::tabs::create_tab_in_active_category(
                params.tabs,
                params.active_tab,
                params.categories,
                params.active_category,
                params.registry,
                params.prompt_registry,
                params.args,
            )
        }
        PendingCommand::NewCategory => {
            crate::framework::widget_system::runtime_loop_helpers::category::create_category_and_tab(
                params.tabs,
                params.active_tab,
                params.categories,
                params.active_category,
                params.registry,
                params.prompt_registry,
                params.args,
            )
        }
        PendingCommand::OpenConversation => {
            crate::framework::widget_system::runtime_loop_helpers::open_conversation::open_conversation_in_tab(
                params.tabs,
                params.active_tab,
                params.categories,
                params.active_category,
                params.registry,
                params.prompt_registry,
                params.args,
            )
        }
        _ => {}
    }
}
