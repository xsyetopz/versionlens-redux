use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use napi::bindgen_prelude::{AsyncTask, Task};
use napi_derive::napi;
use versionlens_core::VersionLensSession;

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
        inner: Arc::new(RwLock::new(Some(VersionLensSession::new(
            config.into_core(),
        )))),
    }
}

#[napi]
impl NativeSession {
    #[napi]
    pub fn analyze_document(&self, input: NativeDocumentInput) -> NativeAnalyzeDocumentOutput {
        self.with_session(NativeAnalyzeDocumentOutput::empty(), |session| {
            NativeAnalyzeDocumentOutput::from_core(session.analyze_document(input.into_core()))
        })
    }

    #[napi]
    pub fn resolve_document(&self, input: NativeDocumentInput) -> AsyncTask<ResolveDocumentTask> {
        AsyncTask::new(ResolveDocumentTask {
            session: Arc::clone(&self.inner),
            input: Some(input),
        })
    }

    #[napi]
    pub fn apply_command(&self, input: NativeApplyCommandInput) -> NativeResolveDocumentOutput {
        self.with_session(NativeResolveDocumentOutput::empty(), |session| {
            let (document, command, dependency_name, selected_version) = input.into_parts();
            NativeResolveDocumentOutput::from_core(session.apply_command_with_selected_version(
                document,
                command.as_deref(),
                dependency_name.as_deref(),
                selected_version.as_deref(),
                &[],
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
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write_guard(&self) -> RwLockWriteGuard<'_, Option<VersionLensSession>> {
        self.inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

pub struct ResolveDocumentTask {
    session: Arc<RwLock<Option<VersionLensSession>>>,
    input: Option<NativeDocumentInput>,
}

impl Task for ResolveDocumentTask {
    type Output = NativeResolveDocumentOutput;
    type JsValue = NativeResolveDocumentOutput;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let Some(input) = self.input.take() else {
            return Ok(NativeResolveDocumentOutput::empty());
        };
        let input = input.into_core();
        let guard = self
            .session
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(match guard.as_ref() {
            Some(session) => {
                NativeResolveDocumentOutput::from_core(session.resolve_document(input))
            }
            None => NativeResolveDocumentOutput::empty(),
        })
    }

    fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

#[cfg(test)]
mod tests;
