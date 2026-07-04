mod line;
mod split;

use crate::model::Dependency;

use line::parse_requirement_line;

pub(crate) use split::split_python_requirement;

pub(crate) fn parse_requirements_txt(text: &str) -> Vec<Dependency> {
    text.lines()
        .enumerate()
        .filter_map(|(line_index, line)| parse_requirement_line(line_index, line))
        .collect()
}

#[cfg(test)]
mod tests;
