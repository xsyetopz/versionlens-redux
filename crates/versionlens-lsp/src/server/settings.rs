//! Reads the settings with `workspace/configuration`. Clients that cannot
//! answer it keep the default settings.

use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use lsp_server::{Connection, Message, Request, RequestId, Response};
use serde_json::Value;
use versionlens_core::Settings;

use super::checking::WorkspaceController;
use super::show_configuration_error;
use crate::state::{self, FolderSettings, VersionLensLspState};

const REQUEST_PREFIX: &str = "versionlens/configuration/";

/// How long the server waits for the first settings before it checks with
/// the defaults, so a slow client does not stop the server.
const FIRST_ANSWER_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default)]
pub(super) struct SettingsRequests {
    next: u64,
    /// The newest request. An older answer is out of date and is dropped.
    latest: Option<RequestId>,
}

impl SettingsRequests {
    /// Asks the client for the settings, if it can answer.
    pub(super) fn request(
        &mut self,
        connection: &Connection,
        state: &VersionLensLspState,
    ) -> Result<()> {
        if !state.client.configuration {
            return Ok(());
        }
        let id = RequestId::from(format!("{REQUEST_PREFIX}{}", self.next));
        self.next += 1;
        self.latest = Some(id.clone());
        connection
            .sender
            .send(Message::Request(Request {
                id,
                method: "workspace/configuration".to_owned(),
                params: state.configuration_params(),
            }))
            .context("failed to request settings")
    }

    /// Asks for the first settings and waits for them, so checking does not
    /// start with the wrong settings. Returns the messages that arrived
    /// meanwhile, in order.
    pub(super) fn first(
        &mut self,
        connection: &Connection,
        state: &mut VersionLensLspState,
    ) -> Result<Vec<Message>> {
        self.request(connection, state)?;
        let mut queued = Vec::new();
        let deadline = Instant::now() + FIRST_ANSWER_TIMEOUT;
        while self.latest.is_some() {
            let Ok(message) = connection.receiver.recv_deadline(deadline) else {
                break;
            };
            match message {
                Message::Response(response) if is_answer(&response.id) => {
                    self.answer(connection, state, None, response)?;
                }
                message => queued.push(message),
            }
        }
        Ok(queued)
    }

    /// Applies an answer if it is the newest one.
    pub(super) fn answer(
        &mut self,
        connection: &Connection,
        state: &mut VersionLensLspState,
        checking: Option<&mut WorkspaceController>,
        response: Response,
    ) -> Result<()> {
        if self.latest.as_ref() != Some(&response.id) {
            return Ok(());
        }
        self.latest = None;
        let sections = match response.response_result {
            Ok(result) => serde_json::from_value::<Vec<Value>>(result).unwrap_or_default(),
            Err(error) => return show_configuration_error(connection, error.message),
        };
        let mut settings = Vec::new();
        for folder in sections.chunks(state::SECTIONS.len()) {
            let versionlens = folder.first().cloned().unwrap_or(Value::Null);
            let config = match Settings::from_value(versionlens) {
                Ok(settings) => settings.session_config(),
                Err(message) => return show_configuration_error(connection, message),
            };
            settings.push(FolderSettings {
                config,
                exclusions: state::workspace_exclusions(folder.get(1).unwrap_or(&Value::Null)),
            });
        }
        let changed = match state.replace_settings(settings) {
            Ok(changed) => changed,
            Err(error) => return show_configuration_error(connection, error.to_string()),
        };
        let Some(checking) = checking else {
            return Ok(());
        };
        if changed {
            checking.restart(state)?;
        }
        Ok(())
    }
}

/// Whether a response answers a settings request.
pub(super) fn is_answer(id: &RequestId) -> bool {
    id.to_string().trim_matches('"').starts_with(REQUEST_PREFIX)
}
