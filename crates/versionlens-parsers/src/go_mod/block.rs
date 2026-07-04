const DIRECTIVES: [&str; 3] = ["require", "replace", "exclude"];

pub(super) type GoModBlock = Option<&'static str>;

pub(super) fn update_go_mod_block(trimmed: &str, current_block: &mut GoModBlock) -> bool {
    if let Some(block) = trimmed.strip_suffix('(').map(str::trim) {
        if let Some(directive) = known_directive(block) {
            *current_block = Some(directive);
        }
        return true;
    }

    if trimmed == ")" {
        *current_block = None;
        return true;
    }

    false
}

pub(super) fn go_mod_dependency_group<'a>(
    trimmed: &'a str,
    current_block: GoModBlock,
) -> Option<(&'static str, &'a str)> {
    if current_block.is_none()
        && let Some((directive, rest)) = trimmed.split_once(' ')
        && let Some(directive) = known_directive(directive)
    {
        return Some((directive, rest.trim()));
    }

    Some((current_block?, trimmed))
}

fn known_directive(candidate: &str) -> Option<&'static str> {
    DIRECTIVES
        .iter()
        .copied()
        .find(|directive| *directive == candidate)
}
