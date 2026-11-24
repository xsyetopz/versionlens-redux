//! The `FILE_TYPES` table, assembled from one module of rows per group of ecosystems.
//! A glob that two rows share belongs to the earlier row, so groups keep their order.

use super::{FileType, row};
use crate::flatten::Format::Text;

mod functional;
mod infra;
mod js;
mod jvm;
mod native;
mod python;
mod shared;
mod tooling;

const GROUPS: &[&[FileType]] = &[
    native::ROWS,
    js::ROWS,
    python::ROWS,
    jvm::ROWS,
    functional::ROWS,
    infra::ROWS,
    tooling::ROWS,
];

const SIZE: usize = {
    let mut total = 0;
    let mut group = 0;
    while group < GROUPS.len() {
        total += GROUPS[group].len();
        group += 1;
    }
    total
};

/// Copies every group, in order, into one array: a `static` cannot join slices any other way.
const fn join() -> [FileType; SIZE] {
    let mut all = [row("", &[], Text, "", &[]); SIZE];
    let mut at = 0;
    let mut group = 0;
    while group < GROUPS.len() {
        let mut item = 0;
        while item < GROUPS[group].len() {
            all[at] = GROUPS[group][item];
            at += 1;
            item += 1;
        }
        group += 1;
    }
    all
}

static JOINED: [FileType; SIZE] = join();

pub(crate) static FILE_TYPES: &[FileType] = &JOINED;
