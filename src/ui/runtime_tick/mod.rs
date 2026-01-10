mod active_frame;
mod code_exec;
mod exec_note;
mod overlays;
mod preheat;
mod stream;
mod tabs;

pub use active_frame::{ActiveFrameData, build_display_messages, prepare_active_frame};
pub use code_exec::update_code_exec_results;
pub use exec_note::build_exec_header_note;
pub use overlays::{sync_code_exec_overlay, sync_file_patch_overlay, sync_question_review_overlay};
pub use preheat::{apply_preheat_results, preheat_inactive_tabs};
pub use stream::collect_stream_events_from_batch;
pub use tabs::{finalize_done_tabs, update_tab_widths};
