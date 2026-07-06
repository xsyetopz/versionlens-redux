use napi::bindgen_prelude::{AsyncTask as NapiAsyncTask, Task as NapiTask};
use std::sync::PoisonError as SyncPoisonError;
use std::sync::{Arc as StdArc, RwLock as StdRwLock};
use versionlens_core::{
    AnalyzeDocumentOutput as CoreAnalyzeDocumentOutput,
    ResolveDocumentOutput as CoreResolveDocumentOutput,
};
mod api;
mod model;

use model::NativeAnalyzeDocumentOutput as AnalyzeOutput;
use model::NativeResolveDocumentOutput as ResolveOutput;

pub use api::{NativeSession, create_session};

#[cfg(test)]
pub(crate) fn leaked_string(contents: String) -> &'static str {
    <Box<str>>::leak(contents.into_boxed_str())
}

pub(crate) fn recover_poison<T>(poisoned: SyncPoisonError<T>) -> T {
    poisoned.into_inner()
}

pub(crate) fn clone_arc<T>(value: &StdArc<T>) -> StdArc<T> {
    value.clone()
}

pub(crate) fn empty_resolve_document_output() -> ResolveOutput {
    model::empty_resolve_document_output()
}

pub(crate) fn resolve_document_output_from_core(
    output: CoreResolveDocumentOutput,
) -> ResolveOutput {
    model::resolve_document_output_from_core(output)
}

pub(crate) fn new_session_cell<T>(value: T) -> StdArc<StdRwLock<T>> {
    std::sync::Arc::new(std::sync::RwLock::new(value))
}

pub(crate) fn async_task<T>(task: T) -> NapiAsyncTask<T>
where
    T: NapiTask + Send + 'static,
{
    <NapiAsyncTask<T>>::new(task)
}

pub(crate) fn empty_analyze_document_output() -> AnalyzeOutput {
    model::empty_analyze_document_output()
}

pub(crate) fn analyze_document_output_from_core(
    output: CoreAnalyzeDocumentOutput,
) -> AnalyzeOutput {
    model::analyze_document_output_from_core(output)
}
