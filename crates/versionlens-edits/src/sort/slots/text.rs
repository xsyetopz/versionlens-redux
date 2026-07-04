mod dependency;
mod slot;

pub(in crate::sort) use dependency::{compare_dependencies, dependency_group};
pub(in crate::sort) use slot::{slot_end_text, slot_text_for};
