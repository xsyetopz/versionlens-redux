//! Every `dependencyProperties` section the VS Code settings of upstream `VersionLens` 1.3.0 and of
//! this fork's multi-crate `master` defaulted to, each with a minimal document that holds one
//! dependency there, so a row that stops reading a section either baseline read fails here.
//! Master's list is a superset of upstream's for every shared setting; the first column names it.

use super::found;

mod functional;
mod js;
mod jvm;
mod master;
mod native;
mod unset;

/// (setting, section, path, document, name, requirement).
type Row = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

fn rows() -> impl Iterator<Item = &'static Row> {
    [
        js::ROWS,
        functional::ROWS,
        native::ROWS,
        jvm::ROWS,
        master::ROWS,
        unset::ROWS,
    ]
    .into_iter()
    .flatten()
}

/// Whether the row's document, read at its path, holds the row's dependency.
fn reads(&(_, _, path, text, name, req): &Row) -> bool {
    found(&format!("/work/{path}"), text)
        .iter()
        .any(|(n, r, _)| (n.as_str(), r.as_str()) == (name, req))
}

#[test]
fn reads_every_dependency_section_upstream_and_master_read() {
    // Arrange
    let all: Vec<&Row> = rows().collect();

    // Act
    let missed: Vec<String> = all
        .iter()
        .filter(|row| !reads(row))
        .map(|(setting, section, path, _, name, req)| {
            format!("{setting}: {section} in {path}, want {name}@{req}")
        })
        .collect();

    // Assert
    assert!(
        missed.is_empty(),
        "{} of {} missed:\n{}",
        missed.len(),
        all.len(),
        missed.join("\n")
    );
}

/// Each baseline file type's sections, read through the row that reads the type now, listed per
/// type so a type the baselines read with no section here fails as surely as a section missed.
#[test]
fn reads_every_baseline_file_types_sections() {
    // Arrange
    let mut types: Vec<&str> = crate::catalog::tests::PRIOR_FILES
        .iter()
        .map(|(_, row)| *row)
        .collect();
    types.sort_unstable();
    types.dedup();

    // Act
    let mut report = Vec::new();
    let mut failed = Vec::new();
    for id in types {
        let read: Vec<String> = rows()
            .filter(|(_, _, path, text, ..)| {
                crate::catalog::file_type_of(&format!("/work/{path}"), text).map(|f| f.id)
                    == Some(id)
            })
            .map(|row| {
                let section = row.1;
                if reads(row) {
                    section.to_owned()
                } else {
                    format!("MISSED {section}")
                }
            })
            .collect();
        if read.is_empty() || read.iter().any(|s| s.starts_with("MISSED")) {
            failed.push(id);
        }
        report.push(format!("{id}: {}", read.join(", ")));
    }
    eprintln!("{}", report.join("\n"));

    // Assert
    assert!(failed.is_empty(), "{failed:?}\n{}", report.join("\n"));
}
