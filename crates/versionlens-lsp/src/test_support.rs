use lsp_types::CodeLens;

pub(crate) fn assert_update_code_lenses(lenses: &[CodeLens]) {
    assert!(!lenses.is_empty());
    assert!(lenses.iter().all(|lens| {
        lens.command.as_ref().is_some_and(|command| {
            !command.title.is_empty()
                && command.command == crate::state::UPDATE_DEPENDENCY_COMMAND
                && command.arguments.as_ref().is_some_and(|arguments| {
                    arguments.len() == 1
                        && arguments[0].get("uri").is_some()
                        && arguments[0].get("generation").is_some()
                })
        })
    }));
}
