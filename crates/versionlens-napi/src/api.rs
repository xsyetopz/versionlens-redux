use napi::Env as NapiEnv;
use napi::Result as NapiResult;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use napi::bindgen_prelude::{AsyncTask, Task};
use napi_derive::napi;
use versionlens_core::{ApplyCommandRequest, VersionLensSession, version_lens_session};

use crate::model::{
    NativeAnalyzeDocumentOutput, NativeApplyCommandInput, NativeDocumentInput,
    NativeResolveDocumentOutput, NativeSessionConfig,
};

#[napi]
pub struct NativeSession {
    inner: Arc<RwLock<Option<VersionLensSession>>>,
}

#[napi]
pub fn create_session(config: NativeSessionConfig) -> NativeSession {
    NativeSession {
        inner: crate::new_session_cell(Some(version_lens_session(config.into_core()))),
    }
}

#[napi]
impl NativeSession {
    #[napi]
    pub fn analyze_document(&self, input: NativeDocumentInput) -> NativeAnalyzeDocumentOutput {
        self.with_session(crate::empty_analyze_document_output(), |session| {
            crate::analyze_document_output_from_core(session.analyze_document(input.into_core()))
        })
    }

    #[napi]
    pub fn resolve_document(&self, input: NativeDocumentInput) -> AsyncTask<ResolveDocumentTask> {
        crate::async_task(ResolveDocumentTask {
            session: crate::clone_arc(&self.inner),
            input: Some(input),
        })
    }

    #[napi]
    pub fn apply_command(&self, input: NativeApplyCommandInput) -> NativeResolveDocumentOutput {
        self.with_session(crate::empty_resolve_document_output(), |session| {
            let (document, command, dependency_name, selected_version) = input.into_parts();
            crate::resolve_document_output_from_core(session.apply_command_with_selected_version(
                ApplyCommandRequest {
                    input: document,
                    command: command.as_deref(),
                    dependency_name: dependency_name.as_deref(),
                    selected_version: selected_version.as_deref(),
                    responses: &[],
                },
            ))
        })
    }

    #[napi]
    pub fn clear_cache(&self) {
        if let Some(session) = self.read_guard().as_ref() {
            session.clear_cache();
        }
    }

    #[napi]
    pub fn dispose_session(&self) {
        self.write_guard().take();
    }
}

impl NativeSession {
    fn with_session<T>(&self, disposed: T, operation: impl FnOnce(&VersionLensSession) -> T) -> T {
        let guard = self.read_guard();
        match guard.as_ref() {
            Some(session) => operation(session),
            None => disposed,
        }
    }

    fn read_guard(&self) -> RwLockReadGuard<'_, Option<VersionLensSession>> {
        self.inner
            .read()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
    }

    fn write_guard(&self) -> RwLockWriteGuard<'_, Option<VersionLensSession>> {
        self.inner
            .write()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
    }
}

pub struct ResolveDocumentTask {
    session: Arc<RwLock<Option<VersionLensSession>>>,
    input: Option<NativeDocumentInput>,
}

impl Task for ResolveDocumentTask {
    type Output = NativeResolveDocumentOutput;
    type JsValue = NativeResolveDocumentOutput;

    fn compute(&mut self) -> NapiResult<Self::Output> {
        let Some(input) = self.input.take() else {
            return Ok(crate::empty_resolve_document_output());
        };
        let input = input.into_core();
        let guard = self
            .session
            .read()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned));
        Ok(match guard.as_ref() {
            Some(session) => {
                crate::resolve_document_output_from_core(session.resolve_document(input))
            }
            None => crate::empty_resolve_document_output(),
        })
    }

    fn resolve(&mut self, _: NapiEnv, output: Self::Output) -> NapiResult<Self::JsValue> {
        Ok(output)
    }
}

#[cfg(test)]
mod tests;
