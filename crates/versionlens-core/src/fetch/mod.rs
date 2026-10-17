mod http;
mod latest;
mod runtime;

pub(crate) use latest::{
    github_action_latest, github_current_ref_is_proven, response_update_choices,
};
