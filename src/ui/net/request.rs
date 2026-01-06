use crate::types::Message;
use crate::ui::events::{LlmEvent, RuntimeEvent, send_llm};
use std::sync::mpsc::Sender;
use std::sync::{Arc, atomic::AtomicBool};
use tokio::runtime::Runtime;

use super::helpers::handle_request_error;
use super::net_logging::build_enabled_tools;
use super::stream::stream_request;
use super::types::LlmStreamRequestParams;

pub(crate) fn request_llm_stream(params: LlmStreamRequestParams) {
    let input = RequestInput::new(RequestConfig {
        base_url: params.base_url.clone(),
        api_key: params.api_key.clone(),
        model: params.model.clone(),
        messages: params.messages.clone(),
        prompts_dir: params.prompts_dir.clone(),
        log_dir: params.log_dir.clone(),
        log_session_id: params.log_session_id.clone(),
        message_index: params.message_index,
        tab: params.tab,
        request_id: params.request_id,
    });
    let enabled = build_enabled_tools(
        params.enable_web_search,
        params.enable_code_exec,
        params.enable_read_file,
        params.enable_read_code,
        params.enable_modify_file,
    );
    run_llm_stream_with_input(input, enabled, params.cancel, params.tx);
}

fn run_llm_stream_with_input(
    input: RequestInput,
    enabled: Vec<&'static str>,
    cancel: Arc<AtomicBool>,
    tx: Sender<RuntimeEvent>,
) {
    let Some(rt) = init_runtime(&tx, input.tab, input.request_id) else {
        return;
    };
    let result = rt.block_on(stream_request(&input, &enabled, &cancel, &tx));
    if let Err(err) = result {
        handle_request_error(&err, &input, &tx);
    }
}

pub(super) struct RequestInput {
    pub(super) base_url: String,
    pub(super) api_key: String,
    pub(super) model: String,
    pub(super) messages: Vec<Message>,
    pub(super) prompts_dir: String,
    pub(super) log_dir: Option<String>,
    pub(super) log_session_id: String,
    pub(super) message_index: usize,
    pub(super) tab: usize,
    pub(super) request_id: u64,
}

struct RequestConfig {
    base_url: String,
    api_key: String,
    model: String,
    messages: Vec<Message>,
    prompts_dir: String,
    log_dir: Option<String>,
    log_session_id: String,
    message_index: usize,
    tab: usize,
    request_id: u64,
}

impl RequestInput {
    fn new(config: RequestConfig) -> Self {
        Self {
            base_url: config.base_url,
            api_key: config.api_key,
            model: config.model,
            messages: config.messages,
            prompts_dir: config.prompts_dir,
            log_dir: config.log_dir,
            log_session_id: config.log_session_id,
            message_index: config.message_index,
            tab: config.tab,
            request_id: config.request_id,
        }
    }
}

fn init_runtime(tx: &Sender<RuntimeEvent>, tab: usize, request_id: u64) -> Option<Runtime> {
    let rt = Runtime::new();
    if rt.is_err() {
        send_llm(
            tx,
            tab,
            request_id,
            LlmEvent::Error("初始化 Tokio 失败".to_string()),
        );
        return None;
    }
    rt.ok()
}
