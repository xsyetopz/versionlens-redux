pub(crate) struct RequirementRange {
    pub(crate) prefix: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) fn operator_requirement_range(
    requirement: &str,
    operators: &[&str],
) -> RequirementRange {
    let trimmed_start = requirement.len() - requirement.trim_start().len();
    let trimmed = &requirement[trimmed_start..];
    for operator in operators {
        if let Some(rest) = trimmed.strip_prefix(operator) {
            let spaces = rest.len() - rest.trim_start().len();
            let start = trimmed_start + operator.len() + spaces;
            return RequirementRange {
                prefix: requirement[..start].to_owned(),
                start,
                end: requirement.trim_end().len(),
            };
        }
    }

    RequirementRange {
        prefix: "".to_owned(),
        start: trimmed_start,
        end: requirement.trim_end().len(),
    }
}

#[cfg(test)]
mod tests;
