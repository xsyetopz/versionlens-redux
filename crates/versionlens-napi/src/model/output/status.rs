use napi_derive::napi;
use versionlens_vscode_model::StatusPayload;

#[napi(object)]
pub struct NativeStatusPayload {
    pub dependency_count: u32,
    pub update_count: u32,
    pub vulnerability_count: u32,
    pub error_count: u32,
    pub no_match_count: u32,
    pub visible: bool,
    pub text: String,
    pub tooltip: String,
}

impl NativeStatusPayload {
    pub(super) fn empty() -> Self {
        Self {
            dependency_count: 0,
            update_count: 0,
            vulnerability_count: 0,
            error_count: 0,
            no_match_count: 0,
            visible: false,
            text: String::new(),
            tooltip: String::new(),
        }
    }
    pub(super) fn from_core(payload: StatusPayload) -> Self {
        Self {
            dependency_count: payload.dependency_count,
            update_count: payload.update_count,
            vulnerability_count: payload.vulnerability_count,
            error_count: payload.error_count,
            no_match_count: payload.no_match_count,
            visible: payload.visible,
            text: payload.text,
            tooltip: payload.tooltip,
        }
    }
}
