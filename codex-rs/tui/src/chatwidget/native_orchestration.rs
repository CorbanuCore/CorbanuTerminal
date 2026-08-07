//! Thin TUI entry points for the upstream native multi-agent runtime.
//!
//! PF Terminal intentionally does not own a parallel pane, worker, or completion
//! protocol here. `/spawn` creates a visible user instruction that the active
//! agent resolves through the canonical `spawn_agent` tool and model catalogue;
//! `/orchestrate` opens the native agent tree.

use super::*;

impl ChatWidget {
    pub(crate) fn open_native_spawn_prompt(&mut self) {
        let tx = self.app_event_tx.clone();
        let view = CustomPromptView::new(
            "Spawn a native agent".to_string(),
            "Describe the bounded task to delegate".to_string(),
            String::new(),
            Some(
                "The active agent will select an authorized capable runtime from the model catalogue."
                    .to_string(),
            ),
            Box::new(move |task| {
                tx.send(AppEvent::SubmitNativeSpawnRequest { task });
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn submit_native_spawn_request(&mut self, task: String) {
        let task = task.trim();
        if task.is_empty() {
            self.add_error_message("A spawn task is required.".to_string());
            return;
        }
        self.submit_user_message(
            format!(
                "Delegate the following bounded task with the native spawn_agent tool. Select the provider, model, and reasoning effort from the canonical catalogue according to task capability, operator policy, and billing; state the chosen route and rationale before spawning. Do not substitute a requested exact runtime. Task: {task}"
            )
            .into(),
        );
    }

    pub(crate) fn open_native_orchestration(&mut self) {
        self.add_info_message(
            "Native agent tree opened. Use /spawn to delegate a new task; select an agent to inspect or control it."
                .to_string(),
            /*hint*/ None,
        );
        self.app_event_tx.send(AppEvent::OpenAgentPicker);
    }
}
