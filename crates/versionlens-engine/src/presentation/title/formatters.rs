mod latest;
mod no_match;
mod status;

pub(super) use latest::latest_title_text;
pub(super) use no_match::no_match_title_text;
pub(super) use status::{
    build_title_text, current_title_text, directory_not_found_title_text, directory_title_text,
    error_title_text, fixed_title_text, invalid_range_title_text, invalid_title_text,
    not_supported_title_text, satisfies_latest_title_text, satisfies_title_text,
    unresolved_title_text, update_title_text,
};
