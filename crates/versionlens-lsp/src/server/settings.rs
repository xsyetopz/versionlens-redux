//! The `versionlens` settings the client pushes or the server pulls, and what changing each one
//! redoes: HTTP clients, lookups, and manifest reads.

use std::time::Instant;

use lsp_types::{MessageType, Uri};
use serde_json::{Value, json};

use super::{Pending, Server};
use crate::config::Settings;

impl Server {
    pub(super) fn pull_config(&mut self) {
        if self.caps.configuration {
            self.request(
                "workspace/configuration",
                json!({ "items": [{ "section": "versionlens" }] }),
                Pending::Config,
            );
        }
    }

    pub(super) fn configure(&mut self, value: &Value) {
        let mut settings = Settings::from_json(value);
        let configured = (settings.enabled, settings.prereleases);
        if configured.0 == self.configured.0 {
            settings.enabled = self.settings.enabled;
        }
        if configured.1 == self.configured.1 {
            settings.prereleases = self.settings.prereleases;
        }
        self.configured = configured;
        if settings == self.settings {
            return;
        }
        let reread = settings.files != self.settings.files
            || settings.file_types != self.settings.file_types
            || settings.nuget_sources != self.settings.nuget_sources
            || settings.properties != self.settings.properties;
        self.env.nuget.clone_from(&settings.nuget_sources);
        let refetch = settings.registries != self.settings.registries
            || settings.authorization != self.settings.authorization
            || settings.github_token != self.settings.github_token
            || settings.providers != self.settings.providers;
        match self.http_for(&settings) {
            Ok(http) => self.http = http,
            Err(error) => self.log(
                MessageType::ERROR,
                format!("could not apply HTTP settings: {error}"),
            ),
        }
        self.settings = settings;
        let s = &self.settings;
        let text = format!(
            "settings applied: enabled {}, prereleases {}, vulnerabilities {}, timeout {:?}, concurrency {}, registry overrides {:?}",
            s.enabled,
            s.prereleases,
            s.vulnerabilities,
            s.timeout,
            s.concurrency,
            s.registries.keys().collect::<Vec<_>>()
        );
        self.log(MessageType::INFO, text);
        if !self.settings.unread.is_empty() {
            let unread = self.settings.unread.join(", ");
            self.log(
                MessageType::WARNING,
                format!("not read, no counterpart in this version: {unread}; see the README"),
            );
        }
        if refetch {
            self.restart_lookups();
        }
        if reread {
            let docs: Vec<(Uri, String, bool)> = self
                .docs
                .values()
                .map(|d| (d.uri.clone(), d.text.clone(), d.open))
                .collect();
            for (uri, text, open) in docs {
                self.open(&uri, text, open);
            }
        }
        self.schedule_all();
        self.dirty = Some(Instant::now());
    }
}
