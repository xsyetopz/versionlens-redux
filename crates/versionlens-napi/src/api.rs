use napi::Env as NapiEnv;
use napi::Result as NapiResult;
use std::sync::{Arc, Mutex, RwLock};

use napi::bindgen_prelude::{AsyncTask, Task};
use napi_derive::napi;
use std::sync::atomic::{AtomicU64, Ordering};
use versionlens_core::{SessionTask, VersionLensSession, version_lens_session};

use crate::binding::{
    NativeAnalyzeDocumentOutput, NativeApplyCommandInput, NativeDocumentInput,
    NativeResolveDocumentOutput, NativeSessionConfig, NativeWorkspaceCheckEvent,
    NativeWorkspaceCheckingInput, NativeWorkspaceGeneration,
};

mod admission;
mod weights;
mod workspace;
use workspace::WorkspaceClient;

#[napi]
pub struct NativeSession {
    inner: Arc<RwLock<Option<Arc<VersionLensSession>>>>,
    generation: Arc<AtomicU64>,
    workspace: Mutex<WorkspaceClient>,
}

#[napi]
pub fn create_session(config: NativeSessionConfig) -> NativeSession {
    NativeSession {
        generation: Arc::new(AtomicU64::new(0)),
        workspace: Mutex::new(WorkspaceClient::default()),
        inner: crate::new_session_cell(Some(Arc::new(version_lens_session(config.into_core())))),
    }
}

#[napi]
pub fn create_session_with_storage(
    config: NativeSessionConfig,
    directory: String,
) -> AsyncTask<CreateSessionTask> {
    crate::async_task(CreateSessionTask {
        config: Some(config),
        directory,
    })
}

pub struct CreateSessionTask {
    config: Option<NativeSessionConfig>,
    directory: String,
}

impl Task for CreateSessionTask {
    type Output = VersionLensSession;
    type JsValue = NativeSession;

    fn compute(&mut self) -> NapiResult<Self::Output> {
        let config = self
            .config
            .take()
            .ok_or_else(|| napi::Error::from_reason("session configuration unavailable"))?;
        version_lens_session(config.into_core())
            .with_persistent_cache(&self.directory)
            .map_err(|error| napi::Error::from_reason(error.to_string()))
    }

    fn resolve(&mut self, _: NapiEnv, session: Self::Output) -> NapiResult<Self::JsValue> {
        Ok(NativeSession {
            generation: Arc::new(AtomicU64::new(0)),
            workspace: Mutex::new(WorkspaceClient::default()),
            inner: crate::new_session_cell(Some(Arc::new(session))),
        })
    }
}

#[napi]
impl NativeSession {
    #[napi]
    pub fn set_workspace_documents(&self, documents: Vec<NativeDocumentInput>) -> NapiResult<bool> {
        let documents = documents
            .into_iter()
            .map(NativeDocumentInput::into_core)
            .collect::<Vec<_>>();
        let guard = self.write_guard();
        let session = guard.clone();
        let changed = guard
            .as_ref()
            .is_some_and(|session| session.set_workspace_documents(documents.clone()));
        if changed {
            self.generation.fetch_add(1, Ordering::AcqRel);
        }
        drop(guard);
        if let Some(session) = session {
            let monitored = self
                .workspace_guard()
                .documents(&session, documents)
                .map_err(napi::Error::from_reason)?;
            if monitored && !changed {
                self.generation.fetch_add(1, Ordering::AcqRel);
            }
            return Ok(changed || monitored);
        }
        Ok(changed)
    }

    #[napi]
    pub fn invalidate_workspace(&self) -> NapiResult<()> {
        let guard = self.write_guard();
        self.generation.fetch_add(1, Ordering::AcqRel);
        if let Some(session) = guard.as_ref() {
            session.invalidate_workspace();
        }
        drop(guard);
        self.workspace_guard()
            .invalidate()
            .map_err(napi::Error::from_reason)
    }

    #[napi]
    pub fn check_workspace(
        &self,
        input: NativeWorkspaceCheckingInput,
    ) -> NapiResult<NativeWorkspaceGeneration> {
        let session = self
            .session()
            .ok_or_else(|| napi::Error::from_reason("session is disposed"))?;
        let (generation, changed) = self
            .workspace_guard()
            .check(&session, input)
            .map_err(napi::Error::from_reason)?;
        if changed {
            self.generation.fetch_add(1, Ordering::AcqRel);
        }
        Ok(NativeWorkspaceGeneration { generation })
    }

    #[napi]
    pub fn take_workspace_events(&self) -> Vec<NativeWorkspaceCheckEvent> {
        self.workspace_guard().take()
    }

    #[napi]
    pub fn workspace_checking_generation(&self) -> Option<NativeWorkspaceGeneration> {
        self.workspace_guard()
            .generation()
            .map(|generation| NativeWorkspaceGeneration { generation })
    }

    #[napi]
    pub fn document_is_fresh(&self, input: NativeDocumentInput) -> bool {
        self.with_session(false, |session| {
            session.document_is_fresh(&input.into_core())
        })
    }

    #[napi]
    pub fn analyze_document(&self, input: NativeDocumentInput) -> NativeAnalyzeDocumentOutput {
        self.with_session(NativeAnalyzeDocumentOutput::default(), |session| {
            crate::analyze_document_output_from_core(session.analyze_document(input.into_core()))
        })
    }

    #[napi]
    pub async fn resolve_document(
        &self,
        input: NativeDocumentInput,
        background: Option<bool>,
    ) -> napi::Result<NativeResolveDocumentOutput> {
        self.resolve_input(TaskInput::Resolve(input, background.unwrap_or(false)))
            .await
    }

    #[napi]
    pub async fn apply_command(
        &self,
        input: NativeApplyCommandInput,
    ) -> napi::Result<NativeResolveDocumentOutput> {
        self.resolve_input(TaskInput::Apply(input)).await
    }

    #[napi]
    pub fn clear_cache(&self) -> NapiResult<()> {
        let guard = self.write_guard();
        self.generation.fetch_add(1, Ordering::AcqRel);
        if let Some(session) = guard.as_ref() {
            session
                .try_clear_cache()
                .map_err(|error| napi::Error::from_reason(error.to_string()))?;
        }
        drop(guard);
        self.workspace_guard()
            .invalidate()
            .map_err(napi::Error::from_reason)?;
        Ok(())
    }

    #[napi]
    pub fn dispose_session(&self) {
        let session = self.write_guard().take();
        self.workspace_guard().close();
        if let Some(session) = session {
            session.cancel_pending_resolutions();
        }
    }
}

impl NativeSession {
    fn workspace_guard(&self) -> std::sync::MutexGuard<'_, WorkspaceClient> {
        self.workspace.lock().unwrap_or_else(crate::recover_poison)
    }
    async fn resolve_input(&self, input: TaskInput) -> NapiResult<NativeResolveDocumentOutput> {
        let _admission = admission::Admission::acquire(&input).map_err(napi::Error::from_reason)?;
        let expected_generation = self.generation.load(Ordering::Acquire);
        let Some(session) = self.session() else {
            return Ok(crate::empty_resolve_document_output());
        };
        let task = match input {
            TaskInput::Resolve(input, true) => SessionTask::Background(input.into_core()),
            TaskInput::Resolve(input, false) => SessionTask::Resolve(input.into_core()),
            TaskInput::Apply(input) => {
                let (document, command, dependency, selected_version) = input.into_parts();
                SessionTask::Command {
                    document,
                    command,
                    dependency,
                    selected_version,
                }
            }
        };
        let output = match session.run_task(task).await {
            Ok(output) => output,
            Err(_)
                if self.generation.load(Ordering::Acquire) != expected_generation
                    || self.session().is_none() =>
            {
                return Ok(crate::empty_resolve_document_output());
            }
            Err(message) => return Err(napi::Error::from_reason(message)),
        };
        if self.generation.load(Ordering::Acquire) != expected_generation
            || self.session().is_none()
        {
            return Ok(crate::empty_resolve_document_output());
        }
        Ok(crate::resolve_document_output_from_core(output))
    }

    fn with_session<T>(&self, disposed: T, operation: impl FnOnce(&VersionLensSession) -> T) -> T {
        match self.session() {
            Some(session) => operation(&session),
            None => disposed,
        }
    }

    fn session(&self) -> Option<Arc<VersionLensSession>> {
        self.inner
            .read()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
            .clone()
    }

    fn write_guard(&self) -> std::sync::RwLockWriteGuard<'_, Option<Arc<VersionLensSession>>> {
        self.inner
            .write()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
    }
}

enum TaskInput {
    Resolve(NativeDocumentInput, bool),
    Apply(NativeApplyCommandInput),
}

#[cfg(test)]
mod tests;
