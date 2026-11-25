use super::*;
use crate::catalog::file_type;
use crate::extract::extract;

fn sorted(path: &str, text: &str, commas: bool) -> String {
    let deps = extract(file_type(path).unwrap(), text);
    let mut out = text.to_owned();
    for run in runs(text, &deps, commas).into_iter().rev() {
        out.replace_range(run.span, &run.sorted);
    }
    out
}

#[test]
fn sorts_json_entries_and_keeps_last_entry_without_comma() {
    let text = "{\n  \"dependencies\": {\n    \"zod\": \"^3.0.0\",\n    \"Axios\": \"^1.0.0\",\n    \"lodash\": \"^4.0.0\"\n  }\n}\n";
    let want = "{\n  \"dependencies\": {\n    \"Axios\": \"^1.0.0\",\n    \"lodash\": \"^4.0.0\",\n    \"zod\": \"^3.0.0\"\n  }\n}\n";
    assert_eq!(sorted("package.json", text, true), want);
}

#[test]
fn sorts_each_block_between_comments_on_its_own() {
    let text = "requests==2.0\nflask==1.0\n# tools\nruff==0.1\nblack==24.1\n";
    let want = "flask==1.0\nrequests==2.0\n# tools\nblack==24.1\nruff==0.1\n";
    assert_eq!(sorted("requirements.txt", text, false), want);
}

#[test]
fn keeps_crlf_line_endings() {
    let text = "b==1.0\r\na==1.0\r\n";
    assert_eq!(
        sorted("requirements.txt", text, false),
        "a==1.0\r\nb==1.0\r\n"
    );
}

#[test]
fn leaves_single_entries_alone() {
    let text = "requests==2.0\n\nflask==1.0\n";
    let deps = extract(file_type("requirements.txt").unwrap(), text);
    assert_eq!(runs(text, &deps, false), []);
}
