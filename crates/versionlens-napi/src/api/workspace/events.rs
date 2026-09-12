use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

use versionlens_core::{WorkspaceCheckEvent, WorkspaceCheckResult};

use crate::binding::NativeWorkspaceCheckEvent;

const MAX_EVENTS: usize = 64;
const MAX_BYTES: usize = 16 * 1024 * 1024;

struct State {
    events: VecDeque<NativeWorkspaceCheckEvent>,
    bytes: usize,
    generation: u64,
    closed: bool,
}

pub(super) struct EventQueue {
    state: Mutex<State>,
    space: Condvar,
}

impl Default for EventQueue {
    fn default() -> Self {
        Self {
            state: Mutex::new(State {
                events: VecDeque::new(),
                bytes: 0,
                generation: 1,
                closed: false,
            }),
            space: Condvar::new(),
        }
    }
}

impl EventQueue {
    pub(super) fn push(&self, event: WorkspaceCheckEvent) {
        let generation = event.generation;
        let mut event = into_native(event);
        if weight(&event) > MAX_BYTES {
            event.document = None;
            event.uri = None;
            event.kind = "failure".to_owned();
            event.message = Some("Workspace result exceeds the delivery size limit".to_owned());
        }
        let bytes = weight(&event);
        let mut state = self.state.lock().unwrap_or_else(crate::recover_poison);
        while !state.closed
            && state.generation == generation
            && (state.events.len() >= MAX_EVENTS || state.bytes.saturating_add(bytes) > MAX_BYTES)
        {
            state = self.space.wait(state).unwrap_or_else(crate::recover_poison);
        }
        if !state.closed && state.generation == generation {
            state.bytes += bytes;
            state.events.push_back(event);
        }
    }

    pub(super) fn take(&self) -> Vec<NativeWorkspaceCheckEvent> {
        let mut state = self.state.lock().unwrap_or_else(crate::recover_poison);
        let events = state.events.drain(..).collect();
        state.bytes = 0;
        drop(state);
        self.space.notify_all();
        events
    }

    pub(super) fn reset(&self, generation: u64) {
        let mut state = self.state.lock().unwrap_or_else(crate::recover_poison);
        state.events.clear();
        state.bytes = 0;
        state.generation = generation;
        drop(state);
        self.space.notify_all();
    }

    pub(super) fn close(&self) {
        let mut state = self.state.lock().unwrap_or_else(crate::recover_poison);
        state.closed = true;
        state.events.clear();
        state.bytes = 0;
        drop(state);
        self.space.notify_all();
    }
}

fn into_native(event: WorkspaceCheckEvent) -> NativeWorkspaceCheckEvent {
    let mut output = NativeWorkspaceCheckEvent {
        generation: event.generation.to_string(),
        kind: "settled".to_owned(),
        document: None,
        uri: None,
        message: None,
    };
    match event.result {
        WorkspaceCheckResult::Document { input, result } => {
            output.kind = "document".to_owned();
            output.document = Some((*input).into());
            output.message = result.err();
        }
        WorkspaceCheckResult::DiscoveryFailure(failure) => {
            output.kind = "failure".to_owned();
            output.uri = failure
                .path
                .as_deref()
                .and_then(versionlens_core::workspace_file_uri);
            output.message = Some(failure.to_string());
        }
        WorkspaceCheckResult::SnapshotLimit { uri, limit } => {
            output.kind = "failure".to_owned();
            output.uri = Some(uri);
            output.message = Some(format!(
                "Workspace documents exceed the {limit}-byte checking limit"
            ));
        }
        WorkspaceCheckResult::DiscoverySettled => output.kind = "discoverySettled".to_owned(),
        WorkspaceCheckResult::Settled => {}
    }
    if let Some(message) = &mut output.message
        && message.len() > 8192
    {
        let end = message.floor_char_boundary(8192);
        message.truncate(end);
        message.push('…');
    }
    output
}

fn weight(event: &NativeWorkspaceCheckEvent) -> usize {
    let document = event
        .document
        .as_ref()
        .map_or(0, super::super::weights::document_weight);
    document
        .saturating_add(event.generation.capacity())
        .saturating_add(event.kind.capacity())
        .saturating_add(event.uri.as_ref().map_or(0, String::capacity))
        .saturating_add(event.message.as_ref().map_or(0, String::capacity))
        .saturating_add(size_of::<NativeWorkspaceCheckEvent>() + 128)
}

#[cfg(test)]
mod tests;
