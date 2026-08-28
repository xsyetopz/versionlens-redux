use lsp_types::CodeLens;

pub(crate) fn assert_display_code_lenses(lenses: &[CodeLens]) {
    assert!(!lenses.is_empty());
    assert!(lenses.iter().all(|lens| {
        lens.command.as_ref().is_some_and(|command| {
            !command.title.is_empty()
                && command.command == crate::state::DISPLAY_CODE_LENS_COMMAND
                && command.arguments.is_none()
        })
    }));
}
