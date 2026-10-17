use versionlens_model::Dependency;

pub(in crate::sort::slots) fn dependency_start_line(dependency: &Dependency) -> u32 {
    let range = &dependency.range;
    range.start.line
}

pub(in crate::sort::slots) fn dependency_end_line(dependency: &Dependency) -> u32 {
    let range = &dependency.range;
    range.end.line
}
