use versionlens_model::Dependency;

mod block;
mod dependency;

use block::{GoModBlock, go_mod_dependency_group, update_go_mod_block};
use dependency::parse_go_mod_dependency;

pub(crate) fn parse_go_mod(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut current_block: GoModBlock = None;

    for (line_index, line) in text.lines().enumerate() {
        if let Some(dependency) = parse_go_mod_line(line_index, line, &mut current_block) {
            dependencies.push(dependency);
        }
    }

    dependencies
}

fn parse_go_mod_line(
    line_index: usize,
    line: &str,
    current_block: &mut GoModBlock,
) -> Option<Dependency> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return None;
    }

    if update_go_mod_block(trimmed, current_block) {
        return None;
    }

    let (group, content) = go_mod_dependency_group(trimmed, *current_block)?;
    parse_go_mod_dependency(line_index, line, group, content)
}

#[cfg(test)]
mod tests;
