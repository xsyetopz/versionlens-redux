use napi::bindgen_prelude::{AsyncTask as NapiAsyncTask, Task as NapiTask};
use std::sync::PoisonError as SyncPoisonError;
use std::sync::{Arc as StdArc, RwLock as StdRwLock};

pub(crate) fn recover_poison<T>(poisoned: SyncPoisonError<T>) -> T {
    poisoned.into_inner()
}

pub(crate) fn clone_arc<T>(value: &StdArc<T>) -> StdArc<T> {
    value.clone()
}

pub(crate) fn new_session_cell<T>(value: T) -> StdArc<StdRwLock<T>> {
    StdArc::new(StdRwLock::new(value))
}

pub(crate) fn async_task<T>(task: T) -> NapiAsyncTask<T>
where
    T: NapiTask + Send + 'static,
{
    <NapiAsyncTask<T>>::new(task)
}
