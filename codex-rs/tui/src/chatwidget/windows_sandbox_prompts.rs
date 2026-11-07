//! Windows sandbox prompts and warning surfaces for `ChatWidget`.

use super::*;
#[cfg(any(target_os = "windows", test))]
use crate::render::renderable::ColumnRenderable;

/// Names the non-admin sandbox's main gaps next to the option that selects it.
#[cfg(any(target_os = "windows", test))]
const LEGACY_SANDBOX_OPTION_DESCRIPTION: &str =
    "Doesn't block reading your files; its network block can be bypassed";

impl ChatWidget {
    #[cfg(any(target_os = "windows", test))]
    pub(super) fn elevated_windows_sandbox_setup_required(&self) -> bool {
        self.windows_sandbox_config.requires_elevated()
            && (self.windows_sandbox_host != crate::app::WindowsSandboxHost::Local
                || !self.windows_sandbox_elevated_setup_complete)
    }

    #[cfg(any(target_os = "windows", test))]
    pub(crate) fn open_windows_sandbox_enable_prompt(
        &mut self,
        preset: ApprovalPreset,
        profile_selection: Option<PermissionProfileSelection>,
    ) {
        use ratatui_macros::line;

        self.session_telemetry.counter(
            "codex.windows_sandbox.elevated_prompt_shown",
            /*inc*/ 1,
            &[],
        );

        let allow_unelevated = self
            .windows_sandbox_config
            .allows(WindowsSandboxSetupMode::Unelevated);
        let setup_choice_is_required =
            !allow_unelevated || self.elevated_windows_sandbox_setup_required();
        let mut header = ColumnRenderable::new();
        header.push(*Box::new(
            crate::terminal_hyperlinks::HyperlinkText::new(if allow_unelevated {
                vec![
                    line!["Set up the Codex agent sandbox to protect your files and control network access. Learn more <https://developers.openai.com/codex/windows>"],
                ]
            } else {
                vec![
                    line!["Your organization requires the default Codex agent sandbox to continue. Set it up to protect your files and control network access."],
                    line!["Learn more <https://developers.openai.com/codex/windows>"],
                ]
            }),
        ));

        let accept_otel = self.session_telemetry.clone();
        let legacy_otel = self.session_telemetry.clone();
        let legacy_preset = preset.clone();
        let legacy_profile_selection = profile_selection.clone();
        let quit_otel = self.session_telemetry.clone();
        let retry_preset = preset.clone();
        let retry_profile_selection = profile_selection.clone();
        let elevated_item = SelectionItem {
            name: "Set up default sandbox (requires Administrator permissions)".to_string(),
            description: None,
            actions: vec![Box::new(move |tx| {
                accept_otel.counter(
                    "codex.windows_sandbox.elevated_prompt_accept",
                    /*inc*/ 1,
                    &[],
                );
                tx.send(AppEvent::BeginWindowsSandboxElevatedSetup {
                    preset: preset.clone(),
                    profile_selection: profile_selection.clone(),
                });
            })],
            dismiss_on_select: true,
            ..Default::default()
        };
        let mut items = if self
            .windows_sandbox_config
            .allows(WindowsSandboxSetupMode::Elevated)
        {
            vec![elevated_item]
        } else {
            Vec::new()
        };
        if allow_unelevated {
            items.push(SelectionItem {
                name: "Use non-admin sandbox (higher risk if prompt injected)".to_string(),
                description: Some(LEGACY_SANDBOX_OPTION_DESCRIPTION.to_string()),
                actions: vec![Box::new(move |tx| {
                    legacy_otel.counter(
                        "codex.windows_sandbox.elevated_prompt_use_legacy",
                        /*inc*/ 1,
                        &[],
                    );
                    tx.send(AppEvent::BeginWindowsSandboxLegacySetup {
                        preset: legacy_preset.clone(),
                        profile_selection: legacy_profile_selection.clone(),
                    });
                })],
                dismiss_on_select: true,
                require_explicit_confirmation: true,
                ..Default::default()
            });
        }
        items.push(SelectionItem {
            name: "Quit".to_string(),
            description: None,
            actions: vec![Box::new(move |tx| {
                quit_otel.counter(
                    "codex.windows_sandbox.elevated_prompt_quit",
                    /*inc*/ 1,
                    &[],
                );
                tx.send(AppEvent::Exit(ExitMode::ShutdownFirst));
            })],
            dismiss_on_select: true,
            ..Default::default()
        });

        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: None,
            footer_hint: Some(standard_popup_hint_line()),
            items,
            header: Box::new(header),
            on_cancel: setup_choice_is_required.then(|| {
                Box::new(move |tx: &AppEventSender| {
                    tx.send(AppEvent::OpenWindowsSandboxEnablePrompt {
                        preset: retry_preset.clone(),
                        profile_selection: retry_profile_selection.clone(),
                    });
                }) as _
            }),
            ..SelectionViewParams::picker()
        });
    }

    #[cfg(all(not(target_os = "windows"), not(test)))]
    pub(crate) fn open_windows_sandbox_enable_prompt(
        &mut self,
        _preset: ApprovalPreset,
        _profile_selection: Option<PermissionProfileSelection>,
    ) {
    }

    #[cfg(any(target_os = "windows", test))]
    pub(crate) fn open_windows_sandbox_fallback_prompt(
        &mut self,
        preset: ApprovalPreset,
        profile_selection: Option<PermissionProfileSelection>,
    ) {
        use ratatui_macros::line;

        let allow_unelevated = self
            .windows_sandbox_config
            .allows(WindowsSandboxSetupMode::Unelevated);
        let setup_choice_is_required =
            !allow_unelevated || self.elevated_windows_sandbox_setup_required();
        let mut lines = Vec::new();
        lines.push(line![
            "Couldn't set up your sandbox with Administrator permissions".bold()
        ]);
        lines.push(line![""]);
        if allow_unelevated {
            lines.push(line![
                "You can still use Corbanu Terminal in a non-admin sandbox. It carries greater risk if prompt injected."
            ]);
            lines.push(line![
                "The non-admin sandbox blocks writes and deletes outside your workspace except in low-integrity folders, but it can't stop commands from reading your files and its network block can be bypassed.".bold()
            ]);
        } else {
            lines.push(line![
                "Your organization requires the default sandbox before Corbanu Terminal can continue."
            ]);
        }
        lines.push(line![
            "Learn more <https://developers.openai.com/codex/windows>"
        ]);

        let mut header = ColumnRenderable::new();
        header.push(crate::terminal_hyperlinks::HyperlinkText::new(lines));

        let elevated_preset = preset.clone();
        let legacy_preset = preset;
        let retry_preset = elevated_preset.clone();
        let retry_profile_selection = profile_selection.clone();
        let elevated_profile_selection = profile_selection.clone();
        let legacy_profile_selection = profile_selection;
        let quit_otel = self.session_telemetry.clone();
        let elevated_item = SelectionItem {
            name: "Try setting up admin sandbox again".to_string(),
            description: None,
            actions: vec![Box::new({
                let otel = self.session_telemetry.clone();
                let preset = elevated_preset;
                move |tx| {
                    otel.counter(
                        "codex.windows_sandbox.fallback_retry_elevated",
                        /*inc*/ 1,
                        &[],
                    );
                    tx.send(AppEvent::BeginWindowsSandboxElevatedSetup {
                        preset: preset.clone(),
                        profile_selection: elevated_profile_selection.clone(),
                    });
                }
            })],
            dismiss_on_select: true,
            ..Default::default()
        };
        let mut items = if self
            .windows_sandbox_config
            .allows(WindowsSandboxSetupMode::Elevated)
        {
            vec![elevated_item]
        } else {
            Vec::new()
        };
        if allow_unelevated {
            items.push(SelectionItem {
                name: "Use Codex with non-admin sandbox".to_string(),
                description: Some(LEGACY_SANDBOX_OPTION_DESCRIPTION.to_string()),
                actions: vec![Box::new({
                    let otel = self.session_telemetry.clone();
                    let preset = legacy_preset;
                    move |tx| {
                        otel.counter(
                            "codex.windows_sandbox.fallback_use_legacy",
                            /*inc*/ 1,
                            &[],
                        );
                        tx.send(AppEvent::BeginWindowsSandboxLegacySetup {
                            preset: preset.clone(),
                            profile_selection: legacy_profile_selection.clone(),
                        });
                    }
                })],
                dismiss_on_select: true,
                require_explicit_confirmation: true,
                ..Default::default()
            });
        }
        items.push(SelectionItem {
            name: "Quit".to_string(),
            description: None,
            actions: vec![Box::new(move |tx| {
                quit_otel.counter(
                    "codex.windows_sandbox.fallback_prompt_quit",
                    /*inc*/ 1,
                    &[],
                );
                tx.send(AppEvent::Exit(ExitMode::ShutdownFirst));
            })],
            dismiss_on_select: true,
            ..Default::default()
        });

        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: None,
            footer_hint: Some(standard_popup_hint_line()),
            items,
            header: Box::new(header),
            on_cancel: setup_choice_is_required.then(|| {
                Box::new(move |tx: &AppEventSender| {
                    tx.send(AppEvent::OpenWindowsSandboxFallbackPrompt {
                        preset: retry_preset.clone(),
                        profile_selection: retry_profile_selection.clone(),
                    });
                }) as _
            }),
            ..SelectionViewParams::picker()
        });
    }

    #[cfg(all(not(target_os = "windows"), not(test)))]
    pub(crate) fn open_windows_sandbox_fallback_prompt(
        &mut self,
        _preset: ApprovalPreset,
        _profile_selection: Option<PermissionProfileSelection>,
    ) {
    }

    #[cfg(any(target_os = "windows", test))]
    pub(crate) fn maybe_prompt_windows_sandbox_enable(&mut self, show_now: bool) {
        let setup_is_required = !self.windows_sandbox_config.is_enabled()
            || self.elevated_windows_sandbox_setup_required();
        if show_now
            && setup_is_required
            && let Some(mut preset) = builtin_approval_presets()
                .into_iter()
                .find(|preset| preset.id == "auto")
        {
            if self.config.active_project.trust_level.is_none()
                && self.config.config_layer_stack.is_projectless()
            {
                preset.approval = self.config.permissions.approval_policy.value();
            }
            self.open_windows_sandbox_enable_prompt(preset, /*profile_selection*/ None);
        }
    }

    #[cfg(any(target_os = "windows", test))]
    pub(crate) fn show_windows_sandbox_setup_status(&mut self) {
        // While elevated sandbox setup runs, prevent typing so the user doesn't
        // accidentally queue messages that will run under an unexpected mode.
        self.bottom_pane.set_composer_input_enabled(
            /*enabled*/ false,
            Some("Input disabled until setup completes.".to_string()),
        );
        self.bottom_pane.reset_status_timer(Duration::ZERO);
        self.bottom_pane.ensure_status_indicator();
        self.bottom_pane
            .set_interrupt_hint_visible(/*visible*/ false);
        self.set_status(
            "Setting up sandbox...".to_string(),
            Some("Hang tight, this may take a few minutes".to_string()),
            StatusDetailsCapitalization::CapitalizeFirst,
            STATUS_DETAILS_DEFAULT_MAX_LINES,
        );
        self.request_redraw();
    }

    #[cfg(not(any(target_os = "windows", test)))]
    #[allow(dead_code)]
    pub(crate) fn show_windows_sandbox_setup_status(&mut self) {}

    #[cfg(any(target_os = "windows", test))]
    pub(crate) fn clear_windows_sandbox_setup_status(&mut self) {
        self.bottom_pane
            .set_composer_input_enabled(/*enabled*/ true, /*placeholder*/ None);
        self.bottom_pane.hide_status_indicator();
        self.request_redraw();
    }

    #[cfg(not(any(target_os = "windows", test)))]
    pub(crate) fn clear_windows_sandbox_setup_status(&mut self) {}

    /// #1506 item 2 (D262): at startup, asks whether to load the `.codex` or
    /// `.agents` folders of this project that appeared while a non-admin
    /// sandbox command ran. Until confirmed, Core ignores them.
    pub(crate) fn maybe_prompt_planted_project_metadata(&mut self) {
        // Unreadable records can't be confirmed; the startup warning says so.
        let codex_config::PlantedProjectMetadata::Known(planted) =
            codex_config::planted_project_metadata()
        else {
            return;
        };
        let cwd = self.config.cwd.as_path();
        let here: Vec<PathBuf> = planted
            .into_iter()
            .filter(|path| {
                path.parent().is_some_and(|parent| {
                    let parent = dunce::canonicalize(parent).unwrap_or_else(|_| parent.into());
                    let cwd = dunce::canonicalize(cwd).unwrap_or_else(|_| cwd.into());
                    cwd.starts_with(&parent)
                })
            })
            .collect();
        if !here.is_empty() {
            self.open_planted_project_metadata_prompt(here);
        }
    }

    pub(crate) fn open_planted_project_metadata_prompt(&mut self, paths: Vec<PathBuf>) {
        use ratatui_macros::line;

        let mut lines = vec![
            line!["Possibly planted project folders".bold()],
            line![""],
            line![
                "These folders appeared while a non-admin Windows sandbox command ran, so a command may have planted them. Corbanu Terminal isn't loading their config, hooks or skills:"
            ],
        ];
        for path in &paths {
            lines.push(line![format!("  {}", path.display()).cyan()]);
        }
        // D283: a recorded folder that isn't `.codex`/`.agents` is one the
        // sandbox couldn't list when a command ended.
        let unlisted = paths.iter().any(|path| {
            !path.file_name().is_some_and(|name| {
                name.eq_ignore_ascii_case(".codex") || name.eq_ignore_ascii_case(".agents")
            })
        });
        if unlisted {
            lines.push(line![
                "A folder other than .codex or .agents couldn't be listed when a command ended, so nothing below it is loaded; make it readable and check it first."
            ]);
        }
        lines.push(line!["Check their contents before you load them.".bold()]);
        let mut header = ColumnRenderable::new();
        header.push(*Box::new(Paragraph::new(lines).wrap(Wrap { trim: false })));

        let items = vec![
            SelectionItem {
                name: "Keep ignoring them".to_string(),
                description: Some("Ask again next time".to_string()),
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Load them".to_string(),
                description: Some("I checked them; load them from the next start".to_string()),
                actions: vec![Box::new(move |tx| {
                    let message = match confirm_planted_project_metadata(&paths) {
                        Ok(()) => "Confirmed. Restart Corbanu Terminal to load them.".to_string(),
                        Err(err) => format!("Couldn't confirm them: {err}"),
                    };
                    tx.send(AppEvent::InsertHistoryCell(Box::new(
                        crate::history_cell::new_info_event(message, /*hint*/ None),
                    )));
                })],
                dismiss_on_select: true,
                ..Default::default()
            },
        ];
        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: None,
            footer_hint: Some(standard_popup_hint_line()),
            items,
            header: Box::new(header),
            ..Default::default()
        });
    }
}

#[cfg(target_os = "windows")]
fn confirm_planted_project_metadata(paths: &[PathBuf]) -> anyhow::Result<()> {
    for path in paths {
        codex_windows_sandbox::confirm_planted_project_metadata(path)?;
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn confirm_planted_project_metadata(_paths: &[PathBuf]) -> anyhow::Result<()> {
    Ok(())
}
