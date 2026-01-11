use crate::framework::widget_system::runtime::runtime_view::{ViewAction, ViewState, apply_view_action};
use crossterm::event::{KeyCode, KeyEvent};

use crate::framework::widget_system::runtime_dispatch::{
    DispatchContext, LayoutContext, apply_model_selection, apply_prompt_selection, cycle_model,
    cycle_model_prev, sync_model_selection, sync_prompt_selection,
};
use crate::framework::widget_system::runtime::state::{PendingCommand, QuestionDecision};
use crate::framework::widget_system::notice::push_notice;
use crate::services::runtime_question_review;

pub(crate) fn handle_view_action_flow(
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
    action: ViewAction,
    key: KeyEvent,
) -> bool {
    if handle_model_cycle(ctx, action) {
        return true;
    }
    if handle_fork_message(ctx, view, action) {
        return true;
    }
    if handle_prompt_sync(ctx, layout, view, key) {
        return true;
    }
    if handle_selection_actions(ctx, action) {
        return true;
    }
    if handle_question_review_actions(ctx, action) {
        return true;
    }
    if handle_apply_view_action(ctx, view, action) {
        return true;
    }
    handle_model_sync(ctx, layout, view, key)
}

fn handle_model_cycle(ctx: &mut DispatchContext<'_>, action: ViewAction) -> bool {
    if !matches!(action, ViewAction::CycleModel | ViewAction::CycleModelPrev) {
        return false;
    }
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        match action {
            ViewAction::CycleModel => cycle_model(ctx.registry, &mut tab_state.app.model_key),
            ViewAction::CycleModelPrev => {
                cycle_model_prev(ctx.registry, &mut tab_state.app.model_key)
            }
            _ => {}
        }
    }
    true
}

fn handle_fork_message(
    ctx: &mut DispatchContext<'_>,
    view: &mut ViewState,
    action: ViewAction,
) -> bool {
    if let ViewAction::ForkMessage(idx) = action {
        if super::super::fork::fork_message_into_new_tab(ctx, idx) {
            view.overlay.close();
        }
        return true;
    }
    false
}

fn handle_prompt_sync(
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
    key: KeyEvent,
) -> bool {
    if key.code == KeyCode::F(5) && view.overlay.is(crate::framework::widget_system::overlay::OverlayKind::Prompt) {
        sync_prompt_selection(view, ctx, layout);
        return true;
    }
    false
}

fn handle_selection_actions(ctx: &mut DispatchContext<'_>, action: ViewAction) -> bool {
    if let ViewAction::SelectModel(idx) = action {
        apply_model_selection(ctx, idx);
        return true;
    }
    if let ViewAction::SelectPrompt(idx) = action {
        apply_prompt_selection(ctx, idx);
        return true;
    }
    false
}

fn handle_question_review_actions(ctx: &mut DispatchContext<'_>, action: ViewAction) -> bool {
    let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) else {
        return false;
    };
    match action {
        ViewAction::QuestionReviewToggle(idx) => {
            runtime_question_review::toggle_question_decision(tab_state, idx)
        }
        ViewAction::QuestionReviewApprove(idx) => runtime_question_review::set_question_decision(tab_state, idx, QuestionDecision::Approved),
        ViewAction::QuestionReviewReject(idx) => runtime_question_review::set_question_decision(tab_state, idx, QuestionDecision::Rejected),
        ViewAction::QuestionReviewApproveAll => {
            runtime_question_review::set_all_decisions(tab_state, QuestionDecision::Approved)
        }
        ViewAction::QuestionReviewRejectAll => {
            runtime_question_review::set_all_decisions(tab_state, QuestionDecision::Rejected)
        }
        ViewAction::QuestionReviewNextModel(idx) => {
            runtime_question_review::cycle_question_model(tab_state, idx, ctx.registry)
        }
        ViewAction::QuestionReviewPrevModel(idx) => {
            runtime_question_review::cycle_question_model_prev(tab_state, idx, ctx.registry)
        }
        ViewAction::QuestionReviewSetAllModel(idx) => {
            let key = runtime_question_review::selected_model_key(tab_state, idx)
                .map(|key: &str| key.to_string());
            if let Some(key) = key {
                runtime_question_review::set_all_models(tab_state, &key)
            } else {
                false
            }
        }
        ViewAction::QuestionReviewSubmit => handle_question_review_submit(tab_state),
        ViewAction::QuestionReviewCancel => {
            tab_state.app.pending_command = Some(PendingCommand::CancelQuestionReview);
            true
        }
        _ => false,
    }
}

fn handle_question_review_submit(tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState) -> bool {
    if !runtime_question_review::all_questions_decided(tab_state) {
        push_notice(&mut tab_state.app, "仍有未确认的问题");
        return true;
    }
    tab_state.app.pending_command = Some(PendingCommand::SubmitQuestionReview);
    true
}

fn handle_apply_view_action(
    ctx: &mut DispatchContext<'_>,
    view: &mut ViewState,
    action: ViewAction,
) -> bool {
    if apply_view_action(
        action,
        ctx.args.show_system_prompt,
        ctx.tabs,
        ctx.active_tab,
        ctx.categories,
        ctx.active_category,
    ) {
        return true;
    }
    if matches!(
        action,
        ViewAction::SelectModel(_) | ViewAction::SelectPrompt(_)
    ) {
        view.overlay.close();
    }
    false
}

fn handle_model_sync(
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
    key: KeyEvent,
) -> bool {
    if key.code == KeyCode::F(4) && view.overlay.is(crate::framework::widget_system::overlay::OverlayKind::Model) {
        sync_model_selection(view, ctx, layout);
        return true;
    }
    false
}
